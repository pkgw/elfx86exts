# 0.3.0 (2018 Oct 16)

- The tool will now print out its best guess as to the minimum Intel CPU
  generation needed to run the analyzed binary, based on an internal table of
  which extensions were introduced in which model. This is a bit approximate
  because the evolution is not strictly linear. Thanks to
  [@apjanke](https://github.com/apjanke) for
  [the contribution](https://github.com/pkgw/elfx86exts/pull/10)! Bumping the
  minor version because this alters the output format.
- Update dependencies, including to
  [capstone](https://crates.io/crates/capstone) 0.5.0.


# 0.2.0 (2018 Jan 23)

- Support MachO/PE binaries as well! Now the name of this tool is super
  misleading! Oh well, it was worth it. Thanks to
  [@reuben](https://github.com/reuben) for
  [the contribution](https://github.com/pkgw/elfx86exts/pull/1)!


# 0.1.0 (2017 Sep 29)

- Initial release.
