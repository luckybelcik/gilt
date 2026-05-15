(identifier) @variable

["var" "const" "fn" "pub" "return" "if" "else" "put" "break" "continue"] @keyword

(function_definition name: (identifier) @function)
(function_call name: (identifier) @function.call)

(variable_declaration type: (identifier) @type)

(parameter type: (identifier) @type)

(function_definition return_type: (identifier) @type)

["{" "}" "(" ")"] @punctuation.bracket

(integer) @number
(float) @number
(boolean) @boolean

(comment) @comment
(tag_comment) @tag
(test_comment) @comment
(test_success_comment) @diff.plus
(test_error_comment) @diff.minus
(test_case_comment) @comment.doc
