.PHONY: example-backends

example:
	python3 -m http.server 3001 --directory examples/static-backends/server1 & \
	python3 -m http.server 3002 --directory examples/static-backends/server2 & \
	python3 -m http.server 3003 --directory examples/static-backends/server3 & \
	wait
