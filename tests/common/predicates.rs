use crate::common::package::Package;

use indoc::formatdoc;

pub fn mit_license_content(year: &str, copyright_holder: &str) -> String {
    formatdoc! {r#"
            Copyright (c) {year} {copyright_holder}

            Permission is hereby granted, free of charge, to any person obtaining a copy of this software and associated documentation files (the "Software"), to deal in the Software without restriction, including without limitation the rights to use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies of the Software, and to permit persons to whom the Software is furnished to do so, subject to the following conditions:
            
            The above copyright notice and this permission notice shall be included in all copies or substantial portions of the Software.
            
            THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.
    "#,
    copyright_holder = copyright_holder
    }
}

pub fn contains_default_mit_license_content() -> predicates::str::ContainsPredicate {
    contains_mit_license_content("<year>", "<copyright holders>")
}

pub fn contains_mit_license_content(
    year: &str,
    copyright_holder: &str,
) -> predicates::str::ContainsPredicate {
    predicates::str::contains(mit_license_content(year, copyright_holder))
}

pub fn overview_count(count: usize) -> predicates::str::ContainsPredicate {
    predicates::str::contains(format!("#o:[{}]", "o".repeat(count)))
}

pub fn licenses_count(count: usize) -> predicates::str::ContainsPredicate {
    predicates::str::contains(format!("#l:[{}]", "l".repeat(count)))
}

pub fn no_licenses_found(package: &Package) -> predicates::str::ContainsPredicate {
    predicates::str::contains(format!(
        "unable to synthesize license expression for '{} {}': \
            no `license` specified, and no license files were found",
        package.name, package.version
    ))
}
