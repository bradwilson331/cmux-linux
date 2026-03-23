// Stub implementations for libraries that ghostty requires but aren't installed
// These allow linking but the features won't work

#include <stddef.h>
#include <stdint.h>

// glslang stubs
void glslang_initialize_process() {}
void glslang_finalize_process() {}
void* glslang_default_resource() { return NULL; }
void* glslang_shader_create(int stage) { return NULL; }
void glslang_shader_delete(void* shader) {}
int glslang_shader_preprocess(void* shader, void* input) { return 0; }
int glslang_shader_parse(void* shader, void* input) { return 0; }
const char* glslang_shader_get_info_log(void* shader) { return ""; }
const char* glslang_shader_get_info_debug_log(void* shader) { return ""; }
void* glslang_program_create() { return NULL; }
void glslang_program_delete(void* program) {}
void glslang_program_add_shader(void* program, void* shader) {}
int glslang_program_link(void* program, int messages) { return 0; }
const char* glslang_program_get_info_log(void* program) { return ""; }
const char* glslang_program_get_info_debug_log(void* program) { return ""; }
void glslang_program_SPIRV_generate(void* program, int stage) {}
size_t glslang_program_SPIRV_get_size(void* program) { return 0; }
void* glslang_program_SPIRV_get_ptr(void* program) { return NULL; }
void glslang_program_SPIRV_get_messages(void* program) {}

// SPIRV-Cross stubs
typedef void* spvc_context;
typedef void* spvc_parsed_ir;
typedef void* spvc_compiler;
typedef void* spvc_compiler_options;
typedef void* spvc_reflected_resource;
typedef void (*spvc_error_callback)(void*, const char*);

int spvc_context_create(spvc_context* context) { *context = NULL; return 0; }
void spvc_context_destroy(spvc_context context) {}
void spvc_context_set_error_callback(spvc_context context, spvc_error_callback cb, void* userdata) {}
int spvc_context_parse_spirv(spvc_context context, const uint32_t* spirv, size_t word_count, spvc_parsed_ir* parsed_ir) { return 0; }
int spvc_context_create_compiler(spvc_context context, int backend, spvc_parsed_ir parsed_ir, int mode, spvc_compiler* compiler) { return 0; }
int spvc_compiler_create_compiler_options(spvc_compiler compiler, spvc_compiler_options* options) { return 0; }
int spvc_compiler_options_set_uint(spvc_compiler_options options, int option, unsigned value) { return 0; }
int spvc_compiler_install_compiler_options(spvc_compiler compiler, spvc_compiler_options options) { return 0; }
int spvc_compiler_compile(spvc_compiler compiler, const char** source) { return 0; }
int spvc_compiler_get_active_interface_variables(spvc_compiler compiler, void* set) { return 0; }
int spvc_compiler_get_decoration(spvc_compiler compiler, int id, int decoration, unsigned* result) { return 0; }
int spvc_compiler_get_type_handle(spvc_compiler compiler, int id, void* type) { return 0; }
int spvc_type_get_num_array_dimensions(void* type) { return 0; }
int spvc_type_get_array_dimension(void* type, unsigned dimension) { return 0; }
int spvc_type_get_basetype(void* type) { return 0; }
int spvc_type_get_vector_size(void* type) { return 0; }
int spvc_type_get_columns(void* type) { return 0; }
int spvc_resources_get_resource_list_for_type(void* resources, int type, const spvc_reflected_resource** list, size_t* count) { return 0; }

// ImGui stubs
typedef void* ImGuiContext;
typedef void* ImFont;
typedef void* ImFontConfig;
typedef void* ImGuiIO;

void ImGui_SetCurrentContext(ImGuiContext* ctx) {}
void ImGui_DestroyContext(ImGuiContext* ctx) {}
ImGuiContext* ImGui_CreateContext(void* atlas) { return NULL; }
ImGuiIO* ImGui_GetIO() { return NULL; }
void ImGui_StyleColorsDark(void* dst) {}
void ImGui_StyleColorsClassic(void* dst) {}
void ImGui_StyleColorsLight(void* dst) {}

// ImGui font stubs
void ImFontConfig_ImFontConfig(ImFontConfig* self) {}
void ImFontConfig_destroy(ImFontConfig* self) {}
ImFont* ImFontAtlas_AddFontDefault(void* self, const ImFontConfig* config) { return NULL; }
ImFont* ImFontAtlas_AddFontFromMemoryTTF(void* self, void* data, int size, float pixel_size, const ImFontConfig* config, const void* ranges) { return NULL; }
void* ImFontAtlas_GetGlyphRangesDefault(void* self) { return NULL; }
void ImFontAtlas_Build(void* self) {}
void ImFontAtlas_GetTexDataAsRGBA32(void* self, unsigned char** pixels, int* width, int* height, int* bpp) {}
void ImFontAtlas_SetTexID(void* self, void* id) {}

// ImGui widget stubs  
int ImGui_Begin(const char* name, int* open, int flags) { return 0; }
void ImGui_End() {}
void ImGui_Text(const char* fmt, ...) {}
void ImGui_TextUnformatted(const char* text, const char* text_end) {}
int ImGui_Button(const char* label, float x, float y) { return 0; }
void ImGui_Separator() {}
int ImGui_BeginChild_Str(const char* id, float x, float y, int flags, int window_flags) { return 0; }
void ImGui_EndChild() {}
void ImGui_SetScrollHereY(float center) {}
int ImGui_InputText(const char* label, char* buf, size_t buf_size, int flags, void* callback, void* user_data) { return 0; }
void ImGui_SameLine(float offset, float spacing) {}
int ImGui_CollapsingHeader_TreeNodeFlags(const char* label, int flags) { return 0; }
int ImGui_TreeNode_Str(const char* label) { return 0; }
void ImGui_TreePop() {}
void ImGui_Indent(float indent) {}
void ImGui_Unindent(float indent) {}
int ImGui_Selectable_Bool(const char* label, int selected, int flags, float x, float y) { return 0; }
void ImGui_SetNextWindowSize(float x, float y, int cond) {}
void ImGui_SetNextWindowPos(float x, float y, int cond, float px, float py) {}
void ImGui_PushStyleVar_Float(int idx, float val) {}
void ImGui_PushStyleVar_Vec2(int idx, float x, float y) {}
void ImGui_PopStyleVar(int count) {}
void ImGui_PushStyleColor_U32(int idx, unsigned int col) {}
void ImGui_PopStyleColor(int count) {}

// More ImGui stubs
void ImGui_NewFrame() {}
void ImGui_Render() {}
void* ImGui_GetDrawData() { return NULL; }
void ImGui_ShowDemoWindow(int* open) {}
float ImGui_GetFrameHeight() { return 0; }
void ImGui_SetNextItemWidth(float width) {}
int ImGui_BeginCombo(const char* label, const char* preview, int flags) { return 0; }
void ImGui_EndCombo() {}
int ImGui_Checkbox(const char* label, int* v) { return 0; }
void ImGui_TextWrapped(const char* fmt, ...) {}
void ImGui_Columns(int count, const char* id, int border) {}
void ImGui_NextColumn() {}

// ImGuiStyle stubs needed by Inspector
typedef struct ImGuiStyle ImGuiStyle;
void ImGuiStyle_ImGuiStyle(ImGuiStyle* self) {}
void ImGuiStyle_ScaleAllSizes(ImGuiStyle* self, float scale_factor) {}
ImGuiStyle* ImGui_GetStyle() { return NULL; }

// ImGuiIO event stubs needed by Inspector
void ImGuiIO_AddMouseButtonEvent(ImGuiIO* self, int button, int down) {}
void ImGuiIO_AddMousePosEvent(ImGuiIO* self, float x, float y) {}
void ImGuiIO_AddMouseWheelEvent(ImGuiIO* self, float h, float v) {}
void ImGuiIO_AddKeyEvent(ImGuiIO* self, int key, int down) {}
void ImGuiIO_AddInputCharactersUTF8(ImGuiIO* self, const char* str) {}
void ImGuiIO_AddFocusEvent(ImGuiIO* self, int focused) {}

// ImGuiTextFilter stubs
typedef struct ImGuiTextFilter ImGuiTextFilter;
int ImGuiTextFilter_PassFilter(ImGuiTextFilter* self, const char* text, const char* text_end) { return 1; }

// GLAD loader stubs for OpenGL context loading
int gladLoaderLoadGLContext(void* context) { return 1; }  // Return 1 for success
void gladLoaderUnloadGLContext(void* context) {}