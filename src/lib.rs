//! Tweep is a parser for the Twee 3 interactive fiction format
//!
//! # What is Twee?
//! Twee is "a text format for marking up the source code of Twine stories." It
//! is an alternate way to produce interactive fiction stories for the [Twine 2]
//! platform, using plain text files instead of a graphical environment. The
//! specification for Twee 3, supported by tweep, can be found [here]
//!
//! # Goals
//! The goal of tweep is to provide a fully standards-compliant Twee 3 parser
//! that provides helpful warning and error messages for bad practices and
//! common mistakes, which can be used as a backend for a compiler as well as
//! other novel applications of the Twee 3 format.
//!
//! # What it's not
//! * A compiler - while a corresponding compiler front end is in the works,
//!   this is not it. tweep only produces rust objects, not html
//! * A Twee v1 or v2 parser - currently, there are no plans for supporting any
//!   version of the Twee specification other than Twee 3
//!
//! # Getting started
//! To use tweep in your Rust project, simply add the following to your
//! Cargo.toml:
//!
//! ```toml
//! [dependencies]
//! tweep = "0.2"
//! ```
//!
//! For basic parsing, the main entry point into tweep is through the [`Story`]
//! struct, which provides utility methods for parsing out a complete story from
//! a `String` or a `Path` representing a file or directory. When given a
//! directory, tweep will parse all files ending in `.tw` or `.twee` and merge
//! them into a single output story. For more advanced parsing, such as if the
//! tags or metadata attached to a special passage is needed, [`StoryPassages`]
//! provides the same interface, but provides [`Passage`] objects in places
//! where usually unnecessary information is stripped out.
//!
//! # Examples
//! ```
//! use tweep::Story;
//! let input = r#":: StoryTitle
//!RustDoc Sample Story
//!
//!:: StoryData
//!{
//!  "ifid": "D674C58C-DEFA-4F70-B7A2-27742230C0FC",
//!  "format": "SugarCube",
//!  "format-version": "2.28.2",
//!  "start": "My Starting Passage",
//!  "tag-colors": {
//!    "tag1": "green",
//!    "tag2": "red",
//!    "tag3": "blue"
//!  },
//!  "zoom": 0.25
//!}
//!
//!:: My Starting Passage [ tag1 tag2 ]
//!This is the starting passage, specified by the start attribute of StoryData.
//!Alternately, we could remove that attribute and rename the passage to Start.
//!
//!It has tags and links to:
//!  [[Another passage]]
//!  [[Here too!|Another passage]]
//!  [[A third passage<-And a different passage]]
//!
//!:: Another passage {"position":"600,400","size":"100,200"}
//!This passage has some metadata attached to it
//!
//!:: A third passage [tag3] { "position": "400,600" }
//!This passage has both tags and metadata. The size attribute of the metadata
//!isn't overridden, so it will be set to the default value.
//!"#.to_string();
//!
//!// Parse the input into an Output<Result<Story, ErrorList>>
//!let out = Story::from_string(input);
//!assert!(!out.has_warnings());
//!
//!// Move the Result out of the Output
//!let (res, _) = out.take();
//!assert!(res.is_ok());
//!
//!// Get the Story object
//!let story = res.ok().unwrap();
//!
//!// StoryTitle and StoryData contents are parsed into special fields
//!assert_eq!(story.title.unwrap(), "RustDoc Sample Story");
//!assert_eq!(story.data.unwrap().ifid, "D674C58C-DEFA-4F70-B7A2-27742230C0FC");
//!
//!// Other passages are parsed into a map, keyed by the passage name
//!assert_eq!(story.passages["My Starting Passage"].tags(), &vec!["tag1", "tag2"]);
//!let metadata = story.passages["A third passage"].metadata();
//!assert_eq!(metadata["size"], "100,100");
//!assert_eq!(metadata["position"], "400,600");
//! ```
//!
//! [Twine 2]: https://twinery.org/
//! [here]: https://github.com/iftechfoundation/twine-specs/blob/master/twee-3-specification.md
//! [`Story`]: struct.Story.html
//! [`StoryPassages`]: struct.StoryPassages.html
//! [`Passage`]: struct.Passage.html

#![warn(missing_docs)]
#![warn(missing_doc_code_examples)]
mod context;
pub use context::Position;
pub use context::PositionKind;
pub use context::FullContext;
pub use context::PartialContext;

mod issues;
pub use issues::Error;
pub use issues::ErrorList;
pub use issues::ErrorType;
pub use issues::Warning;
pub use issues::WarningType;

mod output;
pub use output::Output;

mod passages;
pub use passages::Passage;
pub use passages::PassageContent;
pub use passages::PassageHeader;
pub use passages::ScriptContent;
pub use passages::StoryData;
pub use passages::StoryTitle;
pub use passages::StylesheetContent;
pub use passages::TwineContent;
pub use passages::TwineLink;
pub use passages::TwinePassage;

mod stories;
#[cfg(feature = "full-context")]
pub use stories::CodeMap;
#[cfg(feature = "full-context")]
pub use stories::ContextErrorList;
pub use stories::Story;
pub use stories::StoryPassages;
