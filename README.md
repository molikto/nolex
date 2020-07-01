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


## TODO

* support comments
    * it can be a metadata of `Token`?
    * or a special `Token.id`? this means the parser should handle this, and complicates the AST