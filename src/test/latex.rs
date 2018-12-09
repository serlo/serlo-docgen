use crate::latex::LatexArgs;
use crate::preamble::*;
use crate::Targets;
use serde_yaml;
use structopt::StructOpt;

macro_rules! test_case {
    ($target:path, $name:ident, $ast:expr, $result:expr) => {
        #[test]
        fn $name() {
            let root = serde_yaml::from_str($ast).expect("could not parse test input!");
            let settings = Settings::default();
            let mut res = vec![];
            let args = LatexArgs::from_iter(["test", "test_doc", "src/test/test.anchors"].iter());
            settings
                .targets
                .get("default")
                .expect("no default configuration!")
                .iter()
                .find_map(|c| if let $target(t) = c { Some(t) } else { None })
                .expect("could not find target!")
                .export(&root, &settings, &args, &mut res)
                .expect("export failed!");
            assert_eq!(&String::from_utf8_lossy(&res), $result);
        }
    };
}

test_case!(
    Targets::Latex,
    simple_text,
    "
type: text
position: {}
text: simple plain text äüöß",
    "simple plain text äüöß"
);

test_case!(
    Targets::Latex,
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
    Targets::Latex,
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
    Targets::Latex,
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
    Targets::Latex,
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
    Targets::Latex,
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
    Targets::Latex,
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
    \\label{dGVzdF9kb2MjaGVhZGluZ19jYXB0aW9u}

    some text
"
);

test_case!(
    Targets::Latex,
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
    Targets::Latex,
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
