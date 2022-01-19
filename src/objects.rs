use std::sync::Arc;

use vulkano::buffer::{BufferUsage, TypedBufferAccess, CpuAccessibleBuffer};
use vulkano::command_buffer::{AutoCommandBufferBuilder, PrimaryAutoCommandBuffer};
use vulkano::command_buffer::pool::standard::StandardCommandPoolBuilder;

use crate::VglRenderer;
use crate::DEBUG;

use crate::core::logical_device::VglLogicalDevice;
use crate::core::pipeline::VglPipeline;

pub mod vertex;
use crate::objects::vertex::Vertex;

pub mod triangle;
pub mod rectangle;
pub mod square;

pub struct VglObject {
    vertices: Option<Vec<Vertex>>,
    indices: Option<Vec<u16>>,

    pipeline_id: usize,
}

impl VglObject {
    pub fn with_pipeline(
        mut self,
        pipeline_id: usize,
    ) -> Self {
        self.pipeline_id = pipeline_id;

        self
    }



    fn generate_quadrilateral_indices(
        indices: &mut Vec<u16>,
        increment: usize,
    ) {
        let index_increment = increment as u16 * 4;

        indices
            .extend(vec!
                [
                0 + index_increment,
                1 + index_increment,
                2 + index_increment,
                2 + index_increment,
                3 + index_increment,
                0 + index_increment,
                ].iter().copied()
            );
    }







    pub fn get_vertices(
        &self,
    ) -> &Vec<Vertex> {
        self.vertices.as_ref().unwrap()
    }

    pub fn get_indices(
        &self,
    ) -> &Vec<u16> {
        self.indices.as_ref().unwrap()
    }






    pub fn check_vertices(
        vertices: &Vec<Vertex>,
    ) {
        if DEBUG {
            for vertex in vertices {
                if vertex.position[0] < -1.0 || vertex.position[1] < -1.0 || vertex.position[0] > 1.0 || vertex.position[1] > 1.0 {
                    panic!("Vertex out of range. (help: Make sure that supplied positions are between -1.0 and 1.0)")
                }
            }
        }
    }

    pub fn check_number_of_vertices(
        vertices: &Vec<Vertex>,
        expected_number: usize,
    ) {
        if DEBUG { 
            if vertices.len() % expected_number != 0 { panic!("Supplied objects don't have {} vertices each", expected_number) } 
        }
    }
}

pub struct VulkanObject {
    vertex_buffer: Option<Arc<CpuAccessibleBuffer<[Vertex]>>>,
    index_buffer: Option<Arc<CpuAccessibleBuffer<[u16]>>>,

    pipeline_id: usize,
}

impl VulkanObject {
    pub fn new(
        logical_device: &VglLogicalDevice,
        object: &VglObject,
    ) -> Self {
        let (vertex_buffer, index_buffer) = Self::generate_buffers(
            logical_device,
            object
        );

        Self {
            vertex_buffer,
            index_buffer,

            pipeline_id: object.pipeline_id,
        }
    }



    fn generate_buffers(
        logical_device: &VglLogicalDevice,
        object: &VglObject,
    ) -> (Option<Arc<CpuAccessibleBuffer<[Vertex]>>>, Option<Arc<CpuAccessibleBuffer<[u16]>>>) {
        let vertex_buffer = Self::generate_vertex_buffer(logical_device, object.get_vertices());

        if object.indices.is_some() {
            let index_buffer = Self::generate_index_buffer(logical_device, object.get_indices());
            (vertex_buffer, index_buffer) 
        } else {
            (vertex_buffer, None)
        }
    }


    fn generate_vertex_buffer(
        logical_device: &VglLogicalDevice,
        vertices: &Vec<Vertex>,
    ) -> Option<Arc<CpuAccessibleBuffer<[Vertex]>>> {
        Some(CpuAccessibleBuffer::from_iter(
                logical_device.clone_logical_device(),
                BufferUsage::all(),
                false,
                vertices.iter().cloned(),
        ).unwrap())
    }

    fn generate_index_buffer(
        logical_device: &VglLogicalDevice,
        indices: &Vec<u16>,
    ) -> Option<Arc<CpuAccessibleBuffer<[u16]>>> {
        Some(CpuAccessibleBuffer::from_iter(
                logical_device.clone_logical_device(),
                BufferUsage::index_buffer(),
                false,
                indices.iter().cloned(),
        ).unwrap())
    }





    pub fn draw(
        &mut self,
        command_buffer_builder: &mut AutoCommandBufferBuilder<PrimaryAutoCommandBuffer, StandardCommandPoolBuilder>,
        pipelines: &Vec<VglPipeline>,
    ) {
        command_buffer_builder
            .bind_pipeline_graphics(pipelines[self.pipeline_id].clone_pipeline())
            .bind_vertex_buffers(0, self.get_vertex_buffer());

        if self.index_buffer.is_some() {
            self.draw_indexed(command_buffer_builder);
        } else {
            self.draw_not_indexed(command_buffer_builder);
        }
    }

    pub fn draw_indexed(
        &mut self,
        command_buffer_builder: &mut AutoCommandBufferBuilder<PrimaryAutoCommandBuffer, StandardCommandPoolBuilder>,
    ) {
        command_buffer_builder
            .bind_index_buffer(self.get_index_buffer())
            .draw_indexed(self.get_index_buffer().len() as u32, 1, 0, 0, 0)
            .unwrap();
    }

    pub fn draw_not_indexed(
        &mut self,
        command_buffer_builder: &mut AutoCommandBufferBuilder<PrimaryAutoCommandBuffer, StandardCommandPoolBuilder>,
    ) {
        command_buffer_builder
            .draw(self.get_vertex_buffer().len() as u32, 1, 0, 0)
            .unwrap();
    }




    pub fn get_vertex_buffer(&self) -> Arc<CpuAccessibleBuffer<[Vertex]>> {
        self.vertex_buffer.clone().unwrap()
    }

    pub fn get_index_buffer(&self) -> Arc<CpuAccessibleBuffer<[u16]>> {
        self.index_buffer.clone().unwrap()
    }
}



impl VglRenderer {
    pub fn add_objects(
        &mut self,
        triangle: &VglObject,
    ) {
        self.objects.push(
            VulkanObject::new(&self.logical_device, triangle)
        );
    }
}






#[cfg(test)]
mod tests {
    use crate::objects::vertex::Vertex;
    use crate::objects::VglObject;

    use crate::DEBUG;

    #[test]
    fn vertices_not_multiple_of_three_panics_in_debug_mode() {
        let vertices = vec!
            [
            Vertex { position: [ 0.0, -0.5] },
            Vertex { position: [ 0.5,  0.5] },
            Vertex { position: [-0.5,  0.5] },
            Vertex { position: [ 0.5,  0.5] },
            ];

        let result = std::panic::catch_unwind(|| VglObject::check_number_of_vertices(&vertices, 3));

        assert_eq!(result.is_err(), DEBUG)
    }

    #[test]
    fn vertices_multiple_of_three_does_not_panic() {
        let vertices = vec!
            [
            Vertex { position: [ 0.0, -0.5] },
            Vertex { position: [ 0.5,  0.5] },
            Vertex { position: [-0.5,  0.5] },
            ];

        VglObject::check_number_of_vertices(&vertices, 3);
    }









    #[test]
    fn first_position_in_first_vertex_littler_than_minus_one_panics_in_debug_mode() {
        let vertex = vec!
            [
            Vertex { position: [-1.3,  0.0] },
            ];

        let result = std::panic::catch_unwind(|| VglObject::check_square_parameters(&vertex, &vec![0.01]));

        assert_eq!(result.is_err(), DEBUG)
    }

    #[test]
    fn second_position_in_first_vertex_littler_than_minus_one_panics_in_debug_mode() {
        let vertex = vec!
            [
            Vertex { position: [ 0.0, -1.3] },
            ];

        let result = std::panic::catch_unwind(|| VglObject::check_square_parameters(&vertex, &vec![0.01]));

        assert_eq!(result.is_err(), DEBUG)
    }

    #[test]
    fn first_position_in_first_vertex_bigger_than_one_panics_in_debug_mode() {
        let vertex = vec!
            [
            Vertex { position: [ 1.3,  0.0] },
            ];

        let result = std::panic::catch_unwind(|| VglObject::check_square_parameters(&vertex, &vec![0.01]));

        assert_eq!(result.is_err(), DEBUG)
    }

    #[test]
    fn second_position_in_first_vertex_bigger_than_one_panics_in_debug_mode() {
        let vertex = vec!
            [
            Vertex { position: [ 0.0,  1.3] },
            ];

        let result = std::panic::catch_unwind(|| VglObject::check_square_parameters(&vertex, &vec![0.01]));

        assert_eq!(result.is_err(), DEBUG)
    }

    #[test]
    fn second_vertex_out_of_bounds_panics_in_debug_mode() {
        let vertex = vec!
            [
            Vertex { position: [ 0.0,  0.0] },
            Vertex { position: [-1.3,  0.0] },
            ];

        let result = std::panic::catch_unwind(|| VglObject::check_square_parameters(&vertex, &vec![0.01]));

        assert_eq!(result.is_err(), DEBUG)
    }
}
