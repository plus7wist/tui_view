[![Crate Status](https://img.shields.io/crates/v/tui_view.svg)](https://crates.io/crates/tui_view)
[![Docs Status](https://docs.rs/tui_view/badge.svg)](https://docs.rs/crate/tui_view/)

A reusable and mildly configurable TUI frontend library. 

The library aims to provide a simple way of loading some data into a terminal interface with no frontend programming. It provides basic actions but it is possible to define custom keybindings too.

All the user needs to do is implement the `Opts` trait with one mandatory method on a struct and pass it into the `create_view` function.

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


### Screenshots

![Screenshot_20230623_210129](https://github.com/nonzac/tui_view/assets/134659163/d9cbd3e2-db0a-4463-829c-b22336a18653)
![Screenshot_20230623_210208](https://github.com/nonzac/tui_view/assets/134659163/9b0518df-cbfd-4746-ad81-23350d1f43b8)
![Screenshot_20230623_210222](https://github.com/nonzac/tui_view/assets/134659163/74ec21e0-64c7-4cd0-899d-492ae55738c6)
![Screenshot_20230623_210242](https://github.com/nonzac/tui_view/assets/134659163/3f6c8464-df84-44cb-978b-7bc696ea1a38)
![Screenshot_20230623_210306](https://github.com/nonzac/tui_view/assets/134659163/9b4c144b-608c-4150-8a45-9e9489712ec4)
