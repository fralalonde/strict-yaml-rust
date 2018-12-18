use scanner::*;
use std::collections::HashMap;

#[derive(Clone, Copy, PartialEq, Debug, Eq)]
enum State {
    StreamStart,
    ImplicitDocumentStart,
    DocumentStart,
    DocumentContent,
    DocumentEnd,
    BlockNode,
    // BlockNodeOrIndentlessSequence,
    BlockSequenceFirstEntry,
    BlockSequenceEntry,
    IndentlessSequenceEntry,
    BlockMappingFirstKey,
    BlockMappingKey,
    BlockMappingValue,
    End
}

/// `Event` is used with the low-level event base parsing API,
/// see `EventReceiver` trait.
#[derive(Clone, PartialEq, Debug, Eq)]
pub enum Event {
    /// Reserved for internal use
    Nothing,
    StreamStart,
    StreamEnd,
    DocumentStart,
    DocumentEnd,
    /// Refer to an anchor ID
    /// Value, style, anchor_id, tag
    Scalar(String, TScalarStyle, usize),
    /// Anchor ID
    SequenceStart(usize),
    SequenceEnd,
    /// Anchor ID
    MappingStart(usize),
    MappingEnd
}

impl Event {
    fn empty_scalar() -> Event {
        // a null scalar
        Event::Scalar("".to_owned(), TScalarStyle::Plain, 0)
    }
}

#[derive(Debug)]
pub struct Parser<T> {
    scanner: Scanner<T>,
    states: Vec<State>,
    state: State,
    marks: Vec<Marker>,
    token: Option<Token>,
    current: Option<(Event, Marker)>,
    anchors: HashMap<String, usize>,
    anchor_id: usize,
}


pub trait EventReceiver {
    fn on_event(&mut self, ev: Event);
}


pub trait MarkedEventReceiver {
    fn on_event(&mut self, ev: Event, _mark: Marker);
}

impl<R: EventReceiver> MarkedEventReceiver for R {
    fn on_event(&mut self, ev: Event, _mark: Marker) {
        self.on_event(ev)
    }
}

pub type ParseResult = Result<(Event, Marker), ScanError>;

impl<T: Iterator<Item=char>> Parser<T> {
    pub fn new(src: T) -> Parser<T> {
        Parser {
            scanner: Scanner::new(src),
            states: Vec::new(),
            state: State::StreamStart,
            marks: Vec::new(),
            token: None,
            current: None,

            anchors: HashMap::new(),
            // valid anchor_id starts from 1
            anchor_id: 1,
        }
    }

    pub fn peek(&mut self) -> Result<&(Event, Marker), ScanError> {
        match self.current {
            Some(ref x) => Ok(x),
            None => {
                self.current = Some(self.next()?);
                self.peek()
            }
        }
    }

    pub fn next(&mut self) -> ParseResult {
        match self.current {
            None => self.parse(),
            Some(_) => Ok(self.current.take().unwrap()),
        }
    }

    fn peek_token(&mut self) -> Result<&Token, ScanError> {
        match self.token {
            None => {
                self.token = Some(self.scan_next_token()?);
                Ok(self.token.as_ref().unwrap())
            }
            Some(ref tok) => Ok(tok),
        }
    }

    fn scan_next_token(&mut self) -> Result<Token, ScanError> {
        let token = self.scanner.next();
        match token {
            None => match self.scanner.get_error() {
                None => Err(ScanError::new(self.scanner.mark(), "unexpected eof")),
                Some(e) => Err(e),
            },
            Some(tok) => Ok(tok),
        }
    }

    fn fetch_token(&mut self) -> Token {
        self.token
            .take()
            .expect("fetch_token needs to be preceded by peek_token")
    }

    fn skip(&mut self) {
        self.token = None;
        //self.peek_token();
    }
    fn pop_state(&mut self) {
        self.state = self.states.pop().unwrap()
    }
    fn push_state(&mut self, state: State) {
        self.states.push(state);
    }

    fn parse(&mut self) -> ParseResult {
        if self.state == State::End {
            return Ok((Event::StreamEnd, self.scanner.mark()));
        }
        let (ev, mark) = self.state_machine()?;
        // println!("EV {:?}", ev);
        Ok((ev, mark))
    }

    pub fn load<R: MarkedEventReceiver>(&mut self, recv: &mut R, multi: bool) -> Result<(), ScanError> {
        if !self.scanner.stream_started() {
            let (ev, mark) = self.next()?;
            assert_eq!(ev, Event::StreamStart);
            recv.on_event(ev, mark);
        }

        if self.scanner.stream_ended() {
            // XXX has parsed?
            recv.on_event(Event::StreamEnd, self.scanner.mark());
            return Ok(());
        }
        loop {
            let (ev, mark) = self.next()?;
            if ev == Event::StreamEnd {
                recv.on_event(ev, mark);
                return Ok(());
            }
            // clear anchors before a new document
            self.anchors.clear();
            self.load_document(ev, mark, recv)?;
            if !multi {
                break;
            }
        }
        Ok(())
    }

    fn load_document<R: MarkedEventReceiver>(&mut self, first_ev: Event, mark: Marker, recv: &mut R) -> Result<(), ScanError> {
        assert_eq!(first_ev, Event::DocumentStart);
        recv.on_event(first_ev, mark);

        let (ev, mark) = self.next()?;
        self.load_node(ev, mark, recv)?;

        // DOCUMENT-END is expected.
        let (ev, mark) = self.next()?;
        assert_eq!(ev, Event::DocumentEnd);
        recv.on_event(ev, mark);

        Ok(())
    }

    fn load_node<R: MarkedEventReceiver>(&mut self, first_ev: Event, mark: Marker, recv: &mut R) -> Result<(), ScanError> {
        match first_ev {
            Event::Scalar(..) => {
                recv.on_event(first_ev, mark);
                Ok(())
            }
            Event::SequenceStart(_) => {
                recv.on_event(first_ev, mark);
                self.load_sequence(recv)
            }
            Event::MappingStart(_) => {
                recv.on_event(first_ev, mark);
                self.load_mapping(recv)
            }
            _ => { println!("UNREACHABLE EVENT: {:?}", first_ev);
                unreachable!(); }
        }
    }

    fn load_mapping<R: MarkedEventReceiver>(&mut self, recv: &mut R) -> Result<(), ScanError> {
        let (mut key_ev, mut key_mark) = self.next()?;
        while key_ev != Event::MappingEnd {
            // key
            self.load_node(key_ev, key_mark, recv)?;

            // value
            let (ev, mark) = self.next()?;
            self.load_node(ev, mark, recv)?;

            // next event
            let (ev, mark) = self.next()?;
            key_ev = ev;
            key_mark = mark;

        }
        recv.on_event(key_ev, key_mark);
        Ok(())
    }

    fn load_sequence<R: MarkedEventReceiver>(&mut self, recv: &mut R) -> Result<(), ScanError> {
        let (mut ev, mut mark) = self.next()?;
        while ev != Event::SequenceEnd {
            self.load_node(ev, mark, recv)?;

            // next event
            let (next_ev, next_mark) = self.next()?;
            ev = next_ev;
            mark = next_mark;
        }
        recv.on_event(ev, mark);
        Ok(())
    }

    fn state_machine(&mut self) -> ParseResult {
        // let next_tok = self.peek_token()?;
        // println!("cur_state {:?}, next tok: {:?}", self.state, next_tok);
        match self.state {
            State::StreamStart => self.stream_start(),

            State::ImplicitDocumentStart => self.document_start(true),
            State::DocumentStart => self.document_start(false),
            State::DocumentContent => self.document_content(),
            State::DocumentEnd => self.document_end(),

            State::BlockNode => self.parse_node(true, false),
            // State::BlockNodeOrIndentlessSequence => self.parse_node(true, true),

            State::BlockMappingFirstKey => self.block_mapping_key(true),
            State::BlockMappingKey => self.block_mapping_key(false),
            State::BlockMappingValue => self.block_mapping_value(),

            State::BlockSequenceFirstEntry => self.block_sequence_entry(true),
            State::BlockSequenceEntry => self.block_sequence_entry(false),

            State::IndentlessSequenceEntry => self.indentless_sequence_entry(),

            /* impossible */
            State::End => unreachable!(),
        }
    }

    fn stream_start(&mut self) -> ParseResult {
        match *self.peek_token()? {
            Token(mark, TokenType::StreamStart(_)) => {
                self.state = State::ImplicitDocumentStart;
                self.skip();
                Ok((Event::StreamStart, mark))
            },
            Token(mark, _) => Err(ScanError::new(mark, "did not find expected <stream-start>")),
        }
    }

    fn document_start(&mut self, implicit: bool) -> ParseResult {
        if !implicit {
            while let TokenType::DocumentEnd = self.peek_token()?.1 {
                self.skip();
            }
        }

        match *self.peek_token()? {
            Token(mark, TokenType::StreamEnd) => {
                self.state = State::End;
                self.skip();
                Ok((Event::StreamEnd, mark))
            }
            Token(_, TokenType::VersionDirective(..))
            | Token(_, TokenType::TagDirective(..))
            | Token(_, TokenType::DocumentStart) => {
                // explicit document
                self._explict_document_start()
            }
            Token(mark, _) if implicit => {
                self.parser_process_directives()?;
                self.push_state(State::DocumentEnd);
                self.state = State::BlockNode;
                Ok((Event::DocumentStart, mark))
            }
            _ => {
                // explicit document
                self._explict_document_start()
            }
        }
    }

    fn parser_process_directives(&mut self) -> Result<(), ScanError> {
        loop {
            match self.peek_token()?.1 {
                TokenType::VersionDirective(_, _) => {
                    // XXX parsing with warning according to spec
                    //if major != 1 || minor > 2 {
                    //    return Err(ScanError::new(tok.0,
                    //        "found incompatible YAML document"));
                    //}
                }
                TokenType::TagDirective(..) => {
                    // TODO add tag directive
                }
                _ => break,
            }
            self.skip();
        }
        // TODO tag directive
        Ok(())
    }

    fn _explict_document_start(&mut self) -> ParseResult {
        self.parser_process_directives()?;
        match *try!(self.peek_token()) {
            Token(mark, TokenType::DocumentStart) => {
                self.push_state(State::DocumentEnd);
                self.state = State::DocumentContent;
                self.skip();
                Ok((Event::DocumentStart, mark))
            }
            Token(mark, _) => Err(ScanError::new(mark, "did not find expected <document start>"))
        }
    }

    fn document_content(&mut self) -> ParseResult {
        match *self.peek_token()? {
            Token(mark, TokenType::VersionDirective(..))
            | Token(mark, TokenType::TagDirective(..))
            | Token(mark, TokenType::DocumentStart)
            | Token(mark, TokenType::DocumentEnd)
            | Token(mark, TokenType::StreamEnd) => {
                self.pop_state();
                // empty scalar
                Ok((Event::empty_scalar(), mark))
            },
            _ => {
                self.parse_node(true, false)
            }
        }
    }

    fn document_end(&mut self) -> ParseResult {
        let mut _implicit = true;
        let marker: Marker = match *self.peek_token()? {
            Token(mark, TokenType::DocumentEnd) => {
                self.skip();
                _implicit = false;
                mark
            },
            Token(mark, _) => mark
        };
        
        // TODO tag handling
        self.state = State::DocumentStart;
        Ok((Event::DocumentEnd, marker))
    }

    fn parse_node(&mut self, block: bool, indentless_sequence: bool) -> ParseResult {
        let anchor_id = 0;
        match *self.peek_token()? {
            Token(mark, TokenType::BlockEntry) if indentless_sequence => {
                self.state = State::IndentlessSequenceEntry;
                Ok((Event::SequenceStart(anchor_id), mark))
            },
            Token(_, TokenType::Scalar(..)) => {
                self.pop_state();
                if let Token(mark, TokenType::Scalar(style, v)) = self.fetch_token() {
                    Ok((Event::Scalar(v, style, anchor_id), mark))
                } else {
                    unreachable!()
                }
            },
            Token(mark, TokenType::BlockSequenceStart) if block => {
                self.state = State::BlockSequenceFirstEntry;
                Ok((Event::SequenceStart(anchor_id), mark))
            },
            Token(mark, TokenType::BlockMappingStart) if block => {
                self.state = State::BlockMappingFirstKey;
                Ok((Event::MappingStart(anchor_id), mark))
            },
            Token(mark, _) => { Err(ScanError::new(mark, "while parsing a node, did not find expected node content")) }
        }
    }

    fn block_mapping_key(&mut self, first: bool) -> ParseResult {
        // skip BlockMappingStart
        if first {
            let _ = self.peek_token()?;
            //self.marks.push(tok.0);
            self.skip();
        }
        match *self.peek_token()? {
            Token(_, TokenType::Key) => {
                self.skip();
                match *self.peek_token()? {
                    Token(mark, TokenType::Key)
                    | Token(mark, TokenType::Value)
                    | Token(mark, TokenType::BlockEnd) => {
                        self.state = State::BlockMappingValue;
                        // empty scalar
                        Ok((Event::empty_scalar(), mark))
                    }
                    _ => {
                        self.push_state(State::BlockMappingValue);
                        self.parse_node(true, true)
                    }
                }
            },
            // XXX(chenyh): libyaml failed to parse spec 1.2, ex8.18
            Token(mark, TokenType::Value) => {
                self.state = State::BlockMappingValue;
                Ok((Event::empty_scalar(), mark))
            },
            Token(mark, TokenType::BlockEnd) => {
                self.pop_state();
                self.skip();
                Ok((Event::MappingEnd, mark))
            },
            Token(mark, _) => {
                Err(ScanError::new(mark, "while parsing a block mapping, did not find expected key"))
            }
        }
    }

    fn block_mapping_value(&mut self) -> ParseResult {
        match *self.peek_token()? {
            Token(_, TokenType::Value) => {
                self.skip();
                match *self.peek_token()? {
                    Token(mark, TokenType::Key)
                    | Token(mark, TokenType::Value)
                    | Token(mark, TokenType::BlockEnd) => {
                        self.state = State::BlockMappingKey;
                        // empty scalar
                        Ok((Event::empty_scalar(), mark))
                    },
                    _ => {
                        self.push_state(State::BlockMappingKey);
                        self.parse_node(true, true)
                    }
                }
            },
            Token(mark, _) => {
                self.state = State::BlockMappingKey;
                // empty scalar
                Ok((Event::empty_scalar(), mark))
            }
        }
    }

    fn indentless_sequence_entry(&mut self) -> ParseResult {
        match *self.peek_token()? {
            Token(_, TokenType::BlockEntry) => (),
            Token(mark, _) => {
                self.pop_state();
                return Ok((Event::SequenceEnd, mark));
            }
        }
        self.skip();
        match *self.peek_token()? {
            Token(mark, TokenType::BlockEntry)
            | Token(mark, TokenType::Key)
            | Token(mark, TokenType::Value)
            | Token(mark, TokenType::BlockEnd) => {
                self.state = State::IndentlessSequenceEntry;
                Ok((Event::empty_scalar(), mark))
            },
            _ => {
                self.push_state(State::IndentlessSequenceEntry);
                self.parse_node(true, false)
            }
        }
    }

    fn block_sequence_entry(&mut self, first: bool) -> ParseResult {
        // BLOCK-SEQUENCE-START
        if first {
            let _ = self.peek_token()?;
            //self.marks.push(tok.0);
            self.skip();
        }
        match *self.peek_token()? {
            Token(mark, TokenType::BlockEnd) => {
                self.pop_state();
                self.skip();
                Ok((Event::SequenceEnd, mark))
            },
            Token(_, TokenType::BlockEntry) => {
                self.skip();
                match *self.peek_token()? {
                    Token(mark, TokenType::BlockEntry)
                    | Token(mark, TokenType::BlockEnd) => {
                        self.state = State::BlockSequenceEntry;
                        Ok((Event::empty_scalar(), mark))
                    },
                    _ => {
                        self.push_state(State::BlockSequenceEntry);
                        self.parse_node(true, false)
                    }
                }
            },
            Token(mark, _) => {
                Err(ScanError::new(mark,
                        "while parsing a block collection, did not find expected '-' indicator"))
            }
        }
    }

}

#[cfg(test)]
mod test {
    use super::{Event, Parser};

    #[test]
    fn test_peek_eq_parse() {
        let s = r#"
a0 bb: val
a1: &x
b1: 4
b2: d
a2: 4
a3: [1, 2, 3]
a4:
    - [a1, a2]
    - 2
a5: *x
"#;
        let mut p = Parser::new(s.chars());
        while {
            let event_peek = p.peek().unwrap().clone();
            let event = p.next().unwrap();
            assert_eq!(event, event_peek);
            event.0 != Event::StreamEnd
        } {}
    }
}
