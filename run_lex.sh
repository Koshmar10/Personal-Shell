#!/bin/bash

rustc lexer.rs -o bin/lexer && ./bin/lexer "$@"