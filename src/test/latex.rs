use preamble::*;
use serde_yaml;

macro_rules! test_case {
    ($target:expr, $name:ident, $ast:expr, $result:expr) => {
        #[test]
        fn $name() {
            let root = serde_yaml::from_str($ast).expect("could not parse test input!");
            let settings = Settings::default();
            let target = settings
                .general
                .targets
                .get($target)
                .expect("unknown target!")
                .get_target();
            let mut res = vec![];
            target
                .export(&root, &settings, &[], &mut res)
                .expect("export failed!");
            assert_eq!(&String::from_utf8_lossy(&res), $result);
        }
    };
}

test_case!(
    "latex",
    simple_text,
    "
type: text
position: {}
text: simple plain text äüöß",
    "simple plain text äüöß"
);

test_case!(
    "latex",
    paragraph,
    "
type: paragraph
position: {}
content:
    - type: text
      position: {}
      text: some text",
    "some text
"
);

test_case!(
    "latex",
    paragraph_bold,
    "
type: paragraph
position: {}
content:
    - type: text
      position: {}
      text: \"some text \"
    - type: formatted
      position: {}
      markup: bold
      content:
        - type: text
          position: {}
          text: bold text
    - type: text
      position: {}
      text: \" end par\"",
    "some text \\textbf{bold text} end par
"
);

test_case!(
    "latex",
    italic_text,
    "
type: formatted
position: {}
markup: italic
content:
    - type: text
      position: {}
      text: some text",
    "\\textit{some text}"
);

test_case!(
    "latex",
    bold_text,
    "
type: formatted
position: {}
markup: bold
content:
    - type: text
      position: {}
      text: some text",
    "\\textbf{some text}"
);

test_case!(
    "latex",
    nowiki_text,
    "
type: formatted
position: {}
markup: nowiki
content:
    - type: text
      position: {}
      text: some text",
    "some text"
);

test_case!(
    "latex",
    simple_heading,
    "
type: heading
depth: 1
position: {}
caption:
    - type: text
      position: {}
      text: heading caption
content:
    - type: text
      position: {}
      text: some text",
    "\\section{heading caption}
    \\label{JTNDbm9fZG9jdW1lbnRfbmFtZV9zcGVjaWZpZWQlM0UjaGVhZGluZ19jYXB0aW9u}

    some text
"
);

test_case!(
    "latex",
    simple_ulist,
    "
type: list
position: {}
content:
    - type: listitem
      position: {}
      kind: unordered
      depth: 1
      content:
        - type: text
          position: {}
          text: item content 1",
    "\\begin{itemize}
    \\item item content 1
\\end{itemize}
"
);

test_case!(
    "latex",
    simple_olist,
    "
type: list
position: {}
content:
    - type: listitem
      position: {}
      kind: ordered
      depth: 1
      content:
        - type: text
          position: {}
          text: item content 1
    - type: listitem
      position: {}
      kind: ordered
      depth: 1
      content:
        - type: text
          position: {}
          text: item content 2",
    "\\begin{enumerate}
    \\item item content 1
    \\item item content 2
\\end{enumerate}
"
);
