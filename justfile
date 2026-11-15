alias b := build
alias d := debug
alias r := run

# build program
build: 
  odin build .
# run program
run:
  ./mc
# build and run proram
debug:
  odin build . && ./mc
