//! This module provides a [`serde`] implementation.
//!
//! The functions in this module can be used to serialize data
//! structures to StrictYAML and deserialize a data structure from
//! a StrictYAML document stream.

pub mod de;
pub mod error;
pub mod ser;

pub use de::from_str;
pub use de::from_str_many;
pub use de::from_strict_yaml;
pub use ser::to_strict_yaml;
pub use ser::to_string;
pub use ser::to_string_many;

#[cfg(test)]
mod test {
    use super::*;
    use crate::strict_yaml::StrictYaml;
    use serde::{Deserialize, Serialize};

    #[test]
    fn test_primitive_de_ser() {
        let input = r#"---
|
  foo
  bar
"#;

        let expected = "foo\nbar\n".to_string();

        assert_eq!(expected, from_str::<String>(input).unwrap());
        assert_eq!(input, to_string(&expected).unwrap());

        let input = r#"---
"true"
"#;

        assert_eq!(true, from_str(input).unwrap());
        assert_eq!(input, to_string(&true).unwrap());

        let input = r#"---
"false"
"#;

        assert_eq!(false, from_str(input).unwrap());
        assert_eq!(input, to_string(&false).unwrap());

        let input = r#"---
foobar
"#;

        assert_eq!("foobar".to_string(), from_str::<String>(input).unwrap());
        assert_eq!(input, to_string(&"foobar").unwrap());

        let input = r#"---
"78"
"#;

        assert_eq!(78, from_str(input).unwrap());
        assert_eq!(input, to_string(&78).unwrap());

        let input = r#"---
"-78"
"#;

        assert_eq!(-78, from_str(input).unwrap());
        assert_eq!(input, to_string(&-78).unwrap());

        let input = r#"---
"7.8"
"#;

        assert_eq!(7.8, from_str(input).unwrap());
        assert_eq!(input, to_string(&7.8).unwrap());

        let input = r#"---
"-7.8"
"#;

        assert_eq!(-7.8, from_str(input).unwrap());
        assert_eq!(input, to_string(&-7.8).unwrap());

        let input = r#"---
"%"
"#;

        assert_eq!('%', from_str(input).unwrap());
        assert_eq!(input, to_string(&'%').unwrap());
    }

    #[test]
    fn test_option_de_ser() {
        let input = r#"---
foobar
"#;

        let expected = Some("foobar".to_string());

        assert_eq!(expected, from_str::<Option<String>>(input).unwrap());
        assert_eq!(input, to_string(&expected).unwrap());
    }

    #[test]
    fn test_unit_de_ser() {
        assert_eq!((), from_str("").unwrap());

        let input = r#"---
"#;

        assert_eq!((), from_str(input).unwrap());
        assert_eq!(input, to_string(&()).unwrap());
    }

    #[test]
    fn test_unit_struct_de_ser() {
        #[derive(Debug, Deserialize, PartialEq, Serialize)]
        struct Test;

        let input = r#"---
"#;

        assert_eq!(Test, from_str(input).unwrap());
        assert_eq!(input, to_string(&Test).unwrap());
    }

    #[test]
    fn test_newtype_struct_de_ser() {
        #[derive(Debug, Deserialize, PartialEq, Serialize)]
        struct Test(String);

        let input = r#"---
foobar
"#;

        let expected = Test("foobar".to_string());

        assert_eq!(expected, from_str(input).unwrap());
        assert_eq!(input, to_string(&expected).unwrap());
    }

    #[test]
    fn test_vec_de_ser() {
        let input = r#"---
- foo
- bar
- foobar
"#;

        let expected = vec!["foo", "bar", "foobar"];

        assert_eq!(expected, from_str::<Vec<String>>(input).unwrap());
        assert_eq!(input, to_string(&expected).unwrap());
    }

    #[test]
    fn test_tuple_de_ser() {
        let input = r#"---
- foobar
- "false"
- "8"
"#;

        let expected = ("foobar".to_string(), false, 8);

        assert_eq!(expected, from_str::<(String, bool, u8)>(input).unwrap());
        assert_eq!(input, to_string(&expected).unwrap());
    }

    #[test]
    fn test_tuple_struct_de_ser() {
        #[derive(Debug, Deserialize, PartialEq, Serialize)]
        struct Test(String, bool, u8);

        let input = r#"---
- foobar
- "false"
- "8"
"#;

        let expected = Test("foobar".to_string(), false, 8);

        assert_eq!(expected, from_str(input).unwrap());
        assert_eq!(input, to_string(&expected).unwrap());
    }

    #[test]
    fn test_struct_de_ser() {
        #[derive(Debug, Deserialize, PartialEq, Serialize)]
        #[serde(deny_unknown_fields)]
        struct Test {
            a: String,
            b: usize,
            c: bool,
            d: u8,
        }

        let input = r#"---
a: foo
b: "50"
c: "true"
d: "2"
"#;

        let expected = Test {
            a: "foo".to_string(),
            b: 50,
            c: true,
            d: 2,
        };

        assert_eq!(expected, from_str::<Test>(input).unwrap());
        assert_eq!(input, to_string(&expected).unwrap());
    }

    #[test]
    fn test_complex_struct_de_ser() {
        #[derive(Debug, Deserialize, PartialEq, Serialize)]
        struct Test {
            a: bool,
            b: Vec<Item>,
            c: (u8, u8, bool),
            d: Sub,
        }

        #[derive(Debug, Deserialize, PartialEq, Serialize)]
        struct Item {
            foo: String,
            bar: f64,
        }

        #[derive(Debug, Deserialize, PartialEq, Serialize)]
        struct Sub {
            x: bool,
            y: String,
            z: i64,
        }

        let input = r#"---
a: "false"
b:
  - foo: some value
    bar: "100.1234"
  - foo: other value
    bar: "101.1234"
  - foo: final value
    bar: "102.1234"
c:
  - "10"
  - "12"
  - "false"
d:
  x: "false"
  y: |
    foo
    bar
  z: "6"
"#;

        let expected = Test {
            a: false,
            b: vec![
                Item {
                    foo: "some value".to_string(),
                    bar: 100.1234,
                },
                Item {
                    foo: "other value".to_string(),
                    bar: 101.1234,
                },
                Item {
                    foo: "final value".to_string(),
                    bar: 102.1234,
                },
            ],
            c: (10, 12, false),
            d: Sub {
                z: 6,
                x: false,
                y: "foo\nbar\n".to_string(),
            },
        };

        assert_eq!(expected, from_str(input).unwrap());
        assert_eq!(input, to_string(&expected).unwrap());
    }

    #[test]
    fn test_enum_struct_variant_de_ser() {
        #[derive(Debug, Deserialize, PartialEq, Serialize)]
        struct TestStruct {
            first: TestEnum,
            second: Vec<TestEnum>,
        }

        #[derive(Debug, Deserialize, PartialEq, Serialize)]
        enum TestEnum {
            #[serde(rename = "a")]
            A {
                b: usize,
                #[serde(default, skip_serializing_if = "Option::is_none")]
                c: Option<bool>,
                d: Vec<u8>,
            },
        }

        let input = r#"---
first:
  a:
    b: "50"
    c: "true"
    d:
      - "1"
      - "2"
      - "3"
      - "4"
second:
  - a:
      b: "50"
      d:
        - "1"
        - "2"
  - a:
      b: "50"
      c: "false"
      d:
        - "1"
        - "2"
"#;

        let expected = TestStruct {
            first: TestEnum::A {
                b: 50,
                c: Some(true),
                d: vec![1, 2, 3, 4],
            },
            second: vec![
                TestEnum::A {
                    b: 50,
                    c: None,
                    d: vec![1, 2],
                },
                TestEnum::A {
                    b: 50,
                    c: Some(false),
                    d: vec![1, 2],
                },
            ],
        };

        assert_eq!(expected, from_str::<TestStruct>(input).unwrap());
        assert_eq!(input, to_string(&expected).unwrap());
    }

    #[test]
    fn test_enum_newtype_variant_de_ser() {
        #[derive(Debug, Deserialize, PartialEq, Serialize)]
        struct TestStruct {
            first: TestEnum,
            second: Vec<TestEnum>,
        }

        #[derive(Debug, Deserialize, PartialEq, Serialize)]
        enum TestEnum {
            #[serde(rename = "a")]
            A(Vec<u8>),
            B(String),
        }

        let input = r#"---
first:
  a:
    - "1"
    - "2"
second:
  - B: |
      foo
      bar
  - a:
      - "3"
      - "4"
      - "5"
      - "6"
  - B: "true"
"#;

        let expected = TestStruct {
            first: TestEnum::A(vec![1, 2]),
            second: vec![
                TestEnum::B("foo\nbar\n".to_string()),
                TestEnum::A(vec![3, 4, 5, 6]),
                TestEnum::B("true".to_string()),
            ],
        };

        assert_eq!(expected, from_str::<TestStruct>(input).unwrap());
        assert_eq!(input, to_string(&expected).unwrap());
    }

    #[test]
    fn test_enum_tuple_variant_de_ser() {
        #[derive(Debug, Deserialize, PartialEq, Serialize)]
        enum Test {
            #[serde(rename = "a")]
            A(u8, bool, String),
        }

        let input = r#"---
a:
  - "1"
  - "true"
  - foobar
"#;

        let expected = Test::A(1, true, "foobar".to_string());

        assert_eq!(expected, from_str::<Test>(input).unwrap());
        assert_eq!(input, to_string(&expected).unwrap());
    }

    #[test]
    fn test_enum_unit_variant_de_ser() {
        #[derive(Debug, Deserialize, PartialEq, Serialize)]
        enum Test {
            First,
            Second,
        }

        let input = r#"---
First
"#;

        assert_eq!(Test::First, from_str(input).unwrap());
        assert_eq!(input, to_string(&Test::First).unwrap());

        let input = r#"---
First
---
Second
"#;

        let expected = vec![Test::First, Test::Second];

        assert_eq!(expected, from_str_many::<Vec<Test>>(input).unwrap());
        assert_eq!(input, to_string_many(&expected).unwrap());
    }

    #[test]
    fn test_map_de_ser() {
        let input = r#"---
a: |
  foo
  bar
  baz
b: "50"
c: "true"
d: "2"
"#;

        let yaml = from_str::<StrictYaml>(input).unwrap();

        assert_eq!(input, to_string(&yaml).unwrap());
    }

    #[test]
    fn test_array_de_ser() {
        let input = r#"---
- foo
- "50"
- "true"
- "2"
"#;

        let yaml = from_str::<StrictYaml>(input).unwrap();

        assert_eq!(input, to_string(&yaml).unwrap());
    }

    #[test]
    fn test_complex_de_ser() {
        let input = r#"---
a:
  b:
    c: hello
  d: "{}"
e:
  - f
  - g
  - h: "[]"
    d: "10"
  - a:
      - b
      - c
    d: e
c: b
"#;

        let yaml = from_str::<StrictYaml>(input).unwrap();

        assert_eq!(input, to_string(&yaml).unwrap());
    }

    #[test]
    fn test_nested_map_de_ser() {
        let input = r#"---
a:
  b:
    c:
      d:
        e: f
"#;

        let yaml = from_str::<StrictYaml>(input).unwrap();

        assert_eq!(input, to_string(&yaml).unwrap());
    }

    #[test]
    fn test_nested_array_de_ser() {
        let input = r#"---
a:
  - b
  - - c
    - d
    - - e
      - - f
      - - e
"#;

        let yaml = from_str::<StrictYaml>(input).unwrap();

        assert_eq!(input, to_string(&yaml).unwrap());
    }

    #[test]
    fn test_enum_struct_deeply_nested_de_ser() {
        use std::{collections::HashMap, default::Default};

        #[derive(Debug, Deserialize, PartialEq, Serialize)]
        #[serde(deny_unknown_fields)]
        struct Deployment {
            #[serde(rename = "apiVersion")]
            api_version: String,
            kind: Kind,
            metadata: Metadata,
            spec: DeploymentSpec,
        }

        #[derive(Debug, Deserialize, PartialEq, Serialize)]
        enum Kind {
            Deployment,
            StatefulSet,
            DaemonSet,
        }

        #[derive(Debug, Deserialize, PartialEq, Serialize)]
        #[serde(deny_unknown_fields)]
        struct Metadata {
            #[serde(default, skip_serializing_if = "Option::is_none")]
            name: Option<String>,
            labels: HashMap<String, String>,
            #[serde(default, skip_serializing)]
            iteration: usize,
        }

        #[derive(Debug, Deserialize, PartialEq, Serialize)]
        #[serde(deny_unknown_fields)]
        struct DeploymentSpec {
            #[serde(default)]
            replicas: usize,
            selector: Selector,
            template: Template,
        }

        #[derive(Debug, Deserialize, PartialEq, Serialize)]
        enum Selector {
            #[serde(rename = "matchLabels")]
            ByLabel(HashMap<String, String>),
        }

        #[derive(Debug, Deserialize, PartialEq, Serialize)]
        #[serde(deny_unknown_fields)]
        struct Template {
            metadata: Metadata,
            spec: ContainerSpec,
        }

        #[derive(Debug, Deserialize, PartialEq, Serialize)]
        #[serde(deny_unknown_fields)]
        struct ContainerSpec {
            containers: Vec<Container>,
        }

        #[derive(Debug, Deserialize, PartialEq, Serialize)]
        #[serde(deny_unknown_fields)]
        struct Container {
            name: String,
            image: String,
            ports: Vec<Port>,
        }

        #[derive(Debug, Deserialize, PartialEq, Serialize)]
        enum Port {
            #[serde(rename = "containerPort")]
            ContainerPort(usize),
        }

        let input = r#"---
apiVersion: apps/v1
kind: Deployment
metadata:
  name: nginx-deployment
  labels:
    app: nginx
spec:
  replicas: "3"
  selector:
    matchLabels:
      app: nginx
  template:
    metadata:
      labels:
        app: nginx
    spec:
      containers:
        - name: nginx
          image: "nginx:1.14.2"
          ports:
            - containerPort: "80"
            - containerPort: "443"
"#;

        let expected = Deployment {
            api_version: "apps/v1".to_string(),
            kind: Kind::Deployment,
            metadata: Metadata {
                name: Some("nginx-deployment".to_string()),
                labels: HashMap::from([("app".to_string(), "nginx".to_string())]),
                iteration: Default::default(),
            },
            spec: DeploymentSpec {
                replicas: 3,
                selector: Selector::ByLabel(HashMap::from([(
                    "app".to_string(),
                    "nginx".to_string(),
                )])),
                template: Template {
                    metadata: Metadata {
                        name: None,
                        labels: HashMap::from([("app".to_string(), "nginx".to_string())]),
                        iteration: Default::default(),
                    },
                    spec: ContainerSpec {
                        containers: vec![Container {
                            name: "nginx".to_string(),
                            image: "nginx:1.14.2".to_string(),
                            ports: vec![Port::ContainerPort(80), Port::ContainerPort(443)],
                        }],
                    },
                },
            },
        };

        assert_eq!(expected, from_str(input).unwrap());
        assert_eq!(input, to_string(&expected).unwrap());
    }

    #[test]
    fn test_document_stream_de_ser() {
        let input = r#"---
a: foo
b:
  - |
      test
  - "true"
  - - "1"
    - "2"
c: "true"
---
- a
- b:
    c:
      d: e
      f:
        - "true"
        - |
            1
            2
- foobar
---
the end
"#;

        let yaml = from_str_many::<Vec<StrictYaml>>(input).unwrap();

        assert_eq!(input, to_string_many(&yaml).unwrap());
    }
}
