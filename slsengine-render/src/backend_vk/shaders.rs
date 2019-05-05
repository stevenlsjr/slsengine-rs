use vulkano_shaders;

mod backup_vs {
    vulkano_shaders::shader! {
    ty: "vertex",
        src: "
        #version 450

        layout(location = 0) in Vector2<f32> position;
        
        void main(){
            gl_Position = Vector4<f32>(position, 0.0, 1.0);
        }
        "
    }
}

mod backup_fs {
    vulkano_shaders::shader! {
    ty: "fragment",
    src: "#version 450

    layout(location = 0) out Vector4<f32> out_color;
    
    void main(){
        out_color = Vector4<f32>(1.0, 1.0, 0.0, 1.0);
    }
    "
    }

}
