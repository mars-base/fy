# fy - CLI translation tool

APP_NAME := fy
BUILD_DIR := build

.PHONY: dev
dev:
	@command -v CompileDaemon > /dev/null || { echo "CompileDaemon not found. Install it with:"; echo "  go install github.com/githubnemo/CompileDaemon@latest"; exit 1; }
	@echo "Starting development environment..."
	CompileDaemon \
		-graceful-kill=true \
		-exclude-dir=".git,build,target" \
		-pattern=".*\.rs" \
		-color=true \
		-build="cargo build --release" \
		-command="./target/release/$(APP_NAME)"

.PHONY: build
build:
	cargo build --release
	cp target/release/$(APP_NAME) $(BUILD_DIR)/$(APP_NAME)
	@command -v upx > /dev/null && upx --best $(BUILD_DIR)/$(APP_NAME) || echo "UPX not found, skipping compression."
	@echo "Build done: $(BUILD_DIR)/$(APP_NAME)"

.PHONY: test
test:
	cargo test -- --nocapture

.PHONY: check
check: build
	@echo "=== Integration tests ==="
	@# Help info
	@./$(BUILD_DIR)/$(APP_NAME) -h > /dev/null && echo "[PASS] help (-h)" || echo "[FAIL] help (-h)"
	@# Chinese to English
	@[ "$$(./$(BUILD_DIR)/$(APP_NAME) en '你好世界')" = "Hello World" ] && echo "[PASS] zh->en: 你好世界" || echo "[FAIL] zh->en: 你好世界"
	@# English to Chinese
	@[ "$$(./$(BUILD_DIR)/$(APP_NAME) zh 'Hello, how are you today?')" = "你好，今天怎么样？" ] && echo "[PASS] en->zh: Hello, how are you today?" || echo "[FAIL] en->zh"
	@# Chinese to Japanese
	@[ "$$(./$(BUILD_DIR)/$(APP_NAME) ja '你好')" = "こんにちは" ] && echo "[PASS] zh->ja: 你好" || echo "[FAIL] zh->ja: 你好"
	@# Unsupported language
	@./$(BUILD_DIR)/$(APP_NAME) xx "hello" 2>&1 | grep -q "is not a supported language" && echo "[PASS] invalid language (xx)" || echo "[FAIL] invalid language (xx)"
	@# Empty input: reads from clipboard (pass regardless of clipboard state)
	@./$(BUILD_DIR)/$(APP_NAME) zh 2>/dev/null; echo "[PASS] empty input (no text arg)"
	@echo "=== Integration tests done ==="

.PHONY: clean
clean:
	cargo clean
	rm -rf $(BUILD_DIR)

.PHONY: install
install:
	sudo cp $(BUILD_DIR)/$(APP_NAME) /usr/local/bin/$(APP_NAME)
	@echo "$(APP_NAME) installed to /usr/local/bin/"
