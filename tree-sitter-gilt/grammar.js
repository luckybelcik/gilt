/**
 * @file Safe modding language for Kingdoms Game
 * @author Stargrazer Games
 * @license no
 */

/// <reference types="tree-sitter-cli/dsl" />
// @ts-check

module.exports = grammar({
  name: "gilt",

  extras: ($) => [/\s/, $.comment],

  rules: {
    source_file: ($) => repeat($._statement),

    _statement: ($) =>
      choice(
        $.variable_declaration,
        $.assignment,
        $.put_statement,
        $.break_statement,
        $.continue_statement,
        $.expression_statement,
      ),

    variable_declaration: ($) =>
      seq(
        choice("var", "const"),
        field("name", $.identifier),
        choice(
          seq(
            ":",
            field("type", $.identifier),
            "=",
            field("value", $._expression),
          ),
          seq(":=", field("value", $._expression)),
        ),
        ";",
      ),

    assignment: ($) =>
      seq(field("name", $.identifier), "=", field("value", $._expression), ";"),

    put_statement: ($) => seq("put", field("value", $._expression), ";"),

    break_statement: ($) => seq("break", ";"),

    continue_statement: ($) => seq("continue", ";"),

    block: ($) => seq("{", repeat($._statement), "}"),

    _expression: ($) =>
      choice(
        $.identifier,
        $.integer,
        $.float,
        $.boolean,
        $.binary_expression,
        $.block,
        $.if_statement,
        seq("(", $._expression, ")"),
      ),

    expression_statement: ($) => seq($._expression, ";"),

    binary_expression: ($) =>
      prec.left(
        1,
        seq(
          field("left", $._expression),
          field("operator", choice("+", "-", "==", "!=", "<", ">")),
          field("right", $._expression),
        ),
      ),

    identifier: ($) => /[a-z_][a-zA-Z0-9_]*/,

    integer: ($) => /[-]?[0-9]+/,

    float: ($) => /\d+\.\d+/,

    boolean: ($) => choice("true", "false"),

    comment: ($) => seq("//", /.*/),

    if_statement: ($) =>
      seq(
        "if",
        field("condition", $._expression),
        field("consequence", $.block),
        optional(
          seq("else", field("alternative", choice($.block, $.if_statement))),
        ),
      ),
  },
});
