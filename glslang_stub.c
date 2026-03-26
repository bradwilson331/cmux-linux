// Stub implementations for glslang functions that ghostty requires but we don't have
// These are minimal stubs that should allow linking but shader compilation won't work

#include <stddef.h>

void glslang_initialize_process() {}
void glslang_finalize_process() {}

void* glslang_default_resource() { return NULL; }

void* glslang_shader_create(int stage) { return NULL; }
void glslang_shader_delete(void* shader) {}
int glslang_shader_preprocess(void* shader, void* input) { return 0; }
int glslang_shader_parse(void* shader, void* input) { return 0; }
const char* glslang_shader_get_info_log(void* shader) { return ""; }
const char* glslang_shader_get_info_debug_log(void* shader) { return ""; }
const char* glslang_shader_get_preprocessed_code(void* shader) { return ""; }

void* glslang_program_create() { return NULL; }
void glslang_program_delete(void* program) {}
void glslang_program_add_shader(void* program, void* shader) {}
int glslang_program_link(void* program, int messages) { return 0; }
const char* glslang_program_get_info_log(void* program) { return ""; }
const char* glslang_program_get_info_debug_log(void* program) { return ""; }