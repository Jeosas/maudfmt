

# Display this message
@default:
  just --list

# Run the dev build
@dev $RUST_LOG="debug" $RUST_BACKTRACE="1":
  cargo run

@test file:
  cat "{{file}}" | cargo run -- -s | bat -l rust

maudVersion := "v0.27.0"
update-ast:
  cd src/vendor && curl -O https://raw.githubusercontent.com/lambda-fairy/maud/refs/tags/{{maudVersion}}/maud_macros/src/ast.rs

update-readme-help:
  @awk -i inplace 'BEGIN { in_section = 0 } \
  /^<!-- help start -->/ { \
    in_section = 1; \
    print; \
    print ""; \
    print "```console"; \
    print "$ maudfmt --help"; \
    system("cargo run -- --help"); \
    print "```"; \
    print ""; \
  } \
  /^<!-- help end -->/ { in_section = 0 } \
  !in_section' README.md
