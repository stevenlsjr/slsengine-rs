use vulkano_shaders;

mod backup_vs {
    vulkano_shaders::shader! {
    ty: "vertex",
        src: "
        #version 450

        layout(location = 0) in vec2 position;
        
        void main(){
            gl_Position = vec4(position, 0.0, 1.0);
        }
        "
    }
}

mod backup_fs {
    vulkano_shaders::shader! {
    ty: "fragment",
    src: "#version 450

    layout(location = 0) out vec4 out_color;
    
    void main(){
        out_color = vec4(1.0, 1.0, 0.0, 1.0);
    }
    "
    }

}
