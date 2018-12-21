GLSLANG:=glslc
GLSLFLAGS:=-fauto-map-locations -std=450core

REAL_COPY_DIR=$(realpath $(COPY_DIR))
# FRAG_GLSL=vulkan/flat-shading.frag
# VERT_GLSL=vulkan/flat-shading.vert
IN_DIR=shaders/vulkan
OUT_DIR=shaders/spirv

FRAG_GLSL=$(wildcard $(IN_DIR)/*.frag $(IN_DIR)/**/*.frag)
VERT_GLSL=$(wildcard $(IN_DIR)/*.vert $(IN_DIR)/**/*.vert)
COMP_GLSL=$(wildcard $(IN_DIR)/*.comp $(IN_DIR)/**/*.comp)


FRAG_SPV=$(FRAG_GLSL:$(IN_DIR)/%.frag=$(OUT_DIR)/%.frag.spv)
VERT_SPV=$(VERT_GLSL:$(IN_DIR)/%.vert=$(OUT_DIR)/%.vert.spv)
COMP_SPV=$(COMP_GLSL:$(IN_DIR)/%.comp=$(OUT_DIR)/%.comp.spv)

SPIRV_OUT=$(FRAG_SPV) $(VERT_SPV) $(COMP_SPV)

.PHONY: all clean mkdirs copy-assets



copy-assets:
ifdef COPY_DIR
	@echo "copying assets $(REAL_COPY_DIR)"
else
	@ printf "\e[31mno COPY_DIR specified\e[0m\n"
	@ exit 1
endif


all: $(SPIRV_OUT)
	@echo $(FRAG_GLSL)
	@echo $(FRAG_SPV)


mkdirs:
	@mkdir -p shaders/spirv

clean:
	rm -f $(SPIRV_OUT)

$(OUT_DIR)/%.frag.spv: $(IN_DIR)/%.frag
	$(GLSLANG) $(GLSLFLAGS) -o $@  $^

$(OUT_DIR)/%.vert.spv: $(IN_DIR)/%.vert
	$(GLSLANG) $(GLSLFLAGS) -o $@  $^

$(OUT_DIR)/%.comp.spv: $(IN_DIR)/%.comp
	$(GLSLANG) $(GLSLFLAGS) -o $@  $^


