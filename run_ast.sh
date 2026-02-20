#!/bin/bash

rustc ast.rs -o bin/ast && ./bin/ast "$@"