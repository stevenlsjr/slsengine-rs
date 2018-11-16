GLSLANG:=glslc
GLSLFLAGS:=-fauto-map-locations -std=450core

# FRAG_GLSL=vulkan/flat-shading.frag
# VERT_GLSL=vulkan/flat-shading.vert

FRAG_GLSL=$(wildcard ./vulkan/*.frag ./vulkan/**/*.frag)
VERT_GLSL=$(wildcard ./vulkan/*.vert ./vulkan/**/*.vert)
COMP_GLSL=$(wildcard ./vulkan/*.comp ./vulkan/**/*.comp)


FRAG_SPV=$(FRAG_GLSL:%.frag=%.frag.spv)
VERT_SPV=$(VERT_GLSL:%.vert=%.vert.spv)
COMP_SPV=$(COMP_GLSL:%.comp=%.comp.spv)

SPIRV_OUT=$(FRAG_SPV) $(VERT_SPV) $(COMP_SPV)

.PHONY: all clean

all: $(SPIRV_OUT)

clean:
	rm -f $(SPIRV_OUT)

%.frag.spv: %.frag
	$(GLSLANG) $(GLSLFLAGS) -o $@  $^

%.vert.spv: %.vert
	$(GLSLANG) $(GLSLFLAGS) -o $@  $^

%.comp.spv: %.comp
	$(GLSLANG) $(GLSLFLAGS) -o $@  $^


