get-proto-files:
	@cp -r ../proto-defs/proto/ws_server/*.proto ./public/proto/proto_defs
	@cp -r ../proto-defs/proto/api_service/*.proto ./public/proto/proto_defs
	@cp -r ../grpc-service/proto/*.proto ./public/proto/grpc_services
	@echo "Proto files copied to public/proto directory."

generate-all-types: generate-service-types generate-grpc-types

generate-service-types:
	@cp -r  ../proto-defs/proto/* ./public/proto/proto_defs
	@echo "Generating types make sure that all files are in public/proto directory"
	@protoc -I=./public/proto/proto_defs \
			-I=$(shell pkg-config --variable=prefix protobuf)/include \
			--plugin=protoc-gen-ts=./node_modules/.bin/protoc-gen-ts \
			--ts_out=./src/generated/service_types \
			--ts_opt=generate_dependencies,long_type_number \
			$(shell find ./public/proto/proto_defs/ws_server -name '*.proto')
			$(shell find ./public/proto/proto_defs/api_service -name '*.proto')
	@echo "Types generated successfully."


generate-grpc-types:
	@cp -r ../grpc-service/proto/*.proto ./public/proto/grpc_services
	@echo "Generating types make sure that all files are in public/proto directory"
	@protoc -I=./public/proto/grpc_services \
			-I=$(shell pkg-config --variable=prefix protobuf)/include \
			--plugin=protoc-gen-ts=./node_modules/.bin/protoc-gen-ts \
			--ts_out=./src/generated/grpc_service_types \
			--ts_opt=generate_dependencies,long_type_number \
			$(shell find ./public/proto/grpc_services -name '*.proto')
	@echo "Types generated successfully."
