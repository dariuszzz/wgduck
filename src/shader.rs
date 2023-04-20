#[derive(Clone, PartialEq, Eq, Hash)]
pub enum ShaderModule {
    Single {
        module: *const str,
    },
    Separate {
        vertex: *const str,
        fragment: *const str,
    },
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Shader {
    pub modules: ShaderModule,
    pub vert_entry: String,
    pub frag_entry: String,
}

impl Shader {
    pub fn new(
        vert_path: *const str,
        vert_entry: String,
        frag_path: *const str,
        frag_entry: String,
    ) -> Self {
        match vert_path.eq(&frag_path) {
            true => Self {
                modules: ShaderModule::Single { module: vert_path },
                vert_entry,
                frag_entry,
            },
            false => Self {
                modules: ShaderModule::Separate {
                    vertex: vert_path,
                    fragment: frag_path,
                },
                vert_entry,
                frag_entry,
            },
        }
    }
}
