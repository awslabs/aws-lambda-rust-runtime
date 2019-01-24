#!/usr/bin/env python3

# Generates the LambdaErrorExt implementation for all of the Errors
# in the standard library, excluding unstable APIs and errors that
# require generics.
#
# !! Please note this script is a hacky, short term solution !!
import os
from urllib.request import urlopen
from html.parser import HTMLParser

RUST_ERROR_DOCS = "https://doc.rust-lang.org/std/error/trait.Error.html"
GENERATED_FILE_NAME = "./src/error_ext_impl.rs"
UNSTABLE_APIS = ["std::alloc::AllocErr",
                 "std::alloc::CannotReallocInPlace",
                 "std::char::CharTryFromError",
                 "std::num::TryFromIntError"]
GENERIC_ERRORS = ["std::sync::TryLockError",
                  "std::sync::PoisonError",
                  "std::sync::mpsc::TrySendError",
                  "std::sync::mpsc::SendError",
                  "std::io::IntoInnerError"]


class ErrorHtmlParser(HTMLParser):
    def __init__(self):
        super().__init__()
        self.reset()
        self.errors = []
        self.parsing = False
        self.cur_error = self.empty_error()

    def handle_starttag(self, tag, attrs):
        if self.parsing and tag == "a":
            href = ""
            if len(attrs) == 1:
                href = attrs[0][1]
            else:
                href = attrs[1][1]
            parts = href.split("/")
            cnt = 0
            package = ""
            for part in parts:
                cnt = cnt + 1
                if part == "..":
                    continue
                if cnt == len(parts):
                    break

                package += part + "::"

            if package.endswith("::"):
                package = package[0:len(package) - 2]
            self.cur_error["package"] = package
            self.cur_error["href"] = href

    def empty_error(self):
        return {
            "package": "",
            "name": ""
        }

    def handle_data(self, data):
        if data == " Error for " or data == "impl Error for ":
            self.start_parsing()
        else:
            if self.parsing:
                self.cur_error["name"] = data
                if self.is_valid_error(self.cur_error):
                    self.errors.append(self.cur_error)
                self.cur_error = self.empty_error
                self.parsing = False

    def is_valid_error(self, err):
        if err["name"] == "Box":
            return False
        if ".html" in err["package"]:
            return False
        if err["package"] == "":
            return False
        if err["package"] + "::" + err["name"] in UNSTABLE_APIS:
            return False
        if err["package"] + "::" + err["name"] in GENERIC_ERRORS:
            return False

        return True

    def start_parsing(self):
        if not self.parsing:
            self.parsing = True
            self.cur_error = self.empty_error()
        else:
            if self.cur_error["package"] == "" or self.cur_error["name"] == "":
                print("Starting new error with empty existing error")


res = urlopen(RUST_ERROR_DOCS)
assert res.getcode() == 200, "Could not retrieve Rust error docs"

error_docs_html = res.read()
assert error_docs_html != "", "Empty Error docs"

parser = ErrorHtmlParser()
parser.feed(error_docs_html.decode())

print("found {} valid errors. Beginning code generation to {}".format(
    len(parser.errors), GENERATED_FILE_NAME))

if os.path.isfile(GENERATED_FILE_NAME):
    os.remove(GENERATED_FILE_NAME)

# code gen
with open(GENERATED_FILE_NAME, "a") as f:
    f.write("""// Generated code, DO NOT MODIFY!
// This file contains the implementation of the LambdaErrorExt
// trait for most of the standard library errors as well as the
// implementation of the From trait for the HandlerError struct
// to support the same standard library errors.\n\n""")

    # use statements
    for err in parser.errors:
        f.write("use {}::{};\n".format(err["package"], err["name"]))
    f.write(
        "use crate::{LambdaErrorExt, HandlerError};\n\n")

    # impl for LambdaErrorExt for the standard library errors
    for err in parser.errors:
        f.write("""impl LambdaErrorExt for {} {{
    fn error_type(&self) -> &str {{
        "{}::{}"
    }}
}}\n""".format(err["name"], err["package"], err["name"]))

    # impl From trait for standard library errors to HandlerError
    for err in parser.errors:
        f.write("""impl From<{}> for HandlerError {{
    fn from(e: {}) -> Self {{
        HandlerError::new(e)
    }}
}}\n""".format(err["name"], err["name"]))

    f.close()
