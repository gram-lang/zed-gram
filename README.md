# Gram for Zed

[Gram](https://gram-lang.org) recipes in [Zed](https://zed.dev): highlighting, an outline of the sections, diagnostics, completion, hover, formatting and rename.

- **Highlighting and outline** come from [tree-sitter-gram](https://github.com/gram-lang/tree-sitter-gram), at the rev pinned in `extension.toml`.
- **Everything else** comes from the Gram language server, [`@gram-lang/language-server`](https://www.npmjs.com/package/@gram-lang/language-server). A `gram-language-server` found in your `PATH` is used as is; otherwise the extension installs the latest version from npm and runs it with Zed's own Node. It needs version 1.6.0 or later, and says so if npm only has an older one.
- The extension asks the server for no view (`views: false`): the recipe preview and the run sheet are only in VS Code for now.

## Development

```sh
scripts/queries.sh          # copy queries/zed/*.scm of tree-sitter-gram, at the pinned rev, into languages/gram/
scripts/queries.sh --check  # fail if languages/gram/ is out of date (CI)
cargo build --release --target wasm32-wasip2
```

The queries are generated in tree-sitter-gram from one role table: never edit them here. To move to a new tag of the grammar, change `rev` in `extension.toml`, then run `scripts/queries.sh`. The `rev` stays the same as in the Helix and Neovim configurations of the Gram monorepo (`editors/`).

## Manual test before a submission

The Zed registry asks for a manual test at the exact commit submitted.

1. Build the server of the Gram monorepo (`bun run build`) and put it in your `PATH` as `gram-language-server` (or install `@gram-lang/language-server` ≥ 1.6.0 globally).
2. In Zed, run `zed: install dev extension` and pick this folder. To try a local grammar, point `repository` in `extension.toml` to `file:///path/to/tree-sitter-gram` (and set it back before committing).
3. Open a `.gram` recipe (the `editors/smoke/project/` folder of the monorepo has two) and check:
   - the language is **Gram** and the frontmatter, section titles, ingredients, quantities, timers and comments are colored;
   - the outline (`outline: toggle` or the outline panel; command name to check) lists the sections by their title;
   - `editor: toggle comments` writes `// `;
   - a broken recipe shows a diagnostic; completion after `@` offers ingredients; hover shows an ingredient's database name; `editor: format` formats (command name to check); rename works on a `&reference`;
   - `zed: open log` shows the server started, with no error.
4. Remove `gram-language-server` from the `PATH`, restart Zed: the extension installs the server from npm (only once 1.6.0 is published; before, it shows the "needs 1.6.0" message, which is also worth checking).

## License

MIT
