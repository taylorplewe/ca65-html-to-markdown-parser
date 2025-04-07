# ca65.html to Markdown/JSON Parser
Parse `ca65.html` ([`sgml` version](https://github.com/cc65/cc65/blob/master/doc/ca65.sgml)) from the cc65 toolchain into a JSON of markdown, for [`ca65-lsp`](https://github.com/simonhochrein/ca65-lsp)

The goal of the resulting JSON is for help doc for ca65 keywords (like `.macro`) to be shown, during autocomplete & hover, when using the ca65-lsp language server.

This code uses a `Stream` struct written by Simon Hochrein for the aforementioned `ca65-lsp` project.
