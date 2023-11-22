.ONESHELL:

test:
		cargo test --test "*"

build-release:
		cargo build --release

build-wasm:
		cd ./wasm-binding && wasm-pack build

update-profile:
		cargo run --package profile-gen --bin profile-gen -- -p ~/Downloads/Compressed/FitSDKRelease_21.126.00.zip

clean:
