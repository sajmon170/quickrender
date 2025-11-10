use crate::object::{Object, DataToken};

pub struct Scene {
    pub root: Object,
    camera: Option<Object>
}

impl Scene {
    pub fn new(objs: Vec<Object>) -> Self {
        let camera = objs.iter().find(|obj| match obj.get_data() {
            DataToken::Camera(_) => true,
            _ => false
        }).cloned();

        let root = Object::empty().with_children(objs);
        
        Self {
            root,
            camera
        }
    }
    
    pub fn set_camera(&mut self, camera: Object) {
        self.camera = Some(camera);
    }

    pub fn get_camera_object(&mut self) -> Option<&mut Object> {
        self.camera.as_mut()
    }
}
