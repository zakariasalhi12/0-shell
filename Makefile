build:
	docker build -t 0-shell .

run: build
	docker run -it --rm 0-shell

clean:
	docker rmi 0-shell || true
	cargo clean

.PHONY: build run clean