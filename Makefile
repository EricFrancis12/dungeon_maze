EDITOR_DIR_PATH := ./editor

install_editor:
	cd $(EDITOR_DIR_PATH) && npm i

run_editor:
	cd $(EDITOR_DIR_PATH) && npm run dev
