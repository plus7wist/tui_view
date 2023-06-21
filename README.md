[![Crate Status](https://img.shields.io/crates/v/tui_view.svg)](https://crates.io/crates/tui_view)
[![Docs Status](https://docs.rs/tui_view/badge.svg)](https://docs.rs/crate/tui_view/)

A reusable and mildly configurable TUI frontend library. 

The library is aims to provide a simple way of loading some data into a terminal interface with no frontend programming. It porvides basic actions but it is possible to define custom keybindings too.

The library exposes several constructs that can be used to customize its behaviour as well as passing in data.

[Documentation](https://docs.rs/tui_view/latest/tui_view/)

[An app I'm writing using the library](https://github.com/nonzac/something_like_aur)

Since typing searches, it is not possible to define custom keybindings without modifiers.

### Default keybindings
 - \<C-e\>: Exit
 - \<C-d\>: Scroll content down
 - \<C-u\>: Scroll content up
 - \<C-j\>: Select next dock item
 - \<C-k\>: Select previous dock item
 - \<C-b\>: Toggle dock
 - \<C-p\>: Toggle popup
 - Type to search.
