elf: ELF parsing and loading
============================

a library for parsing, navigating, and loading 32- and 64-bit [Executable and Linkable File](http://wiki.osdev.org/ELF)s.

### why are we writing our own ELF lib?

although there are a number of other nice [ELF](https://github.com/nrc/xmas-elf) [libraries](https://github.com/m4b/goblin) for Rust, I'm writing my own for the following handful of reasons:

+ requirements specific to our use-case (no std, no allocation, &c)
+ up to date with recent Rust language features (not all ELF libs are actively maintained)
+ compatibility with SOS' types & representations without translation layers (e.g., uses our `PAddr` type.)
+ it's fun & I want to do it myself!
