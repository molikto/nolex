# nolex

* normally a compiler
    * source file is in `plain text`, so how user input it is irrelevant, any text editor will do
    * source file is `lex` and `parse` to `ast`
    * then later phases happens, like elaboration, type checking, binary generation
* the idea of [structural editor](https://en.wikipedia.org/wiki/Structure_editor) typically
    * user input is converted to `ast operations` **somehow**, this needs a special editor
    * source file is the `ast`
    * later phases...
* this project wants to 
    * user input is 



## creating grammar

ideally the grammar is created within nolex, but because we internally use Tree-Sitter, the workflow now is:

* write your grammar in tree-sitter, all terminal token replace by single letter unicode, no exras
* role assignment functions: basically we need to know where to break in the trees

## code

* as an MVP this only supports JSON now, because generalization takes time, also because we need to use TreeSitter as incremental parsing engine,
so it is hard to generate the source code at runtime.
* our use of TreeSitter it a hack, we only use it to parse but not lex, but these two part of code needs to keep in sync.

## TODO

* support comments
    * it can be a metadata of `Token`?
    * or a special `Token.id`? this means the parser should handle this, and complicates the AST
    
## log

* trivially render all tokens in one line
* port my layouting algorithm!