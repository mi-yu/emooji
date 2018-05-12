build : 
	@cargo build -q

clean :
	@rm -f *.s
	@cargo clean

%.moo : build
	@cargo run -- $*
