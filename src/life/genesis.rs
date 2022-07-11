use std::any::TypeId;
use std::collections::HashMap;

use crate::life::{ComponentPool, Scene};

impl Scene {
    pub fn new() -> Self {
        Self {
            available_entities: vec![0],
            component_pools: HashMap::new(),
        }
    }

    /* Entity */

    pub fn entity(&mut self) -> usize {
        if self.available_entities.len() == 1 {
            self.generate_new_entity()
        } else {
            self.use_recycled_entity()
        }
    }

    pub fn generate_new_entity(&mut self) -> usize {
        let id = self.available_entities[0];
        self.available_entities[0] += 1;

        id
    }

    pub fn use_recycled_entity(&mut self) -> usize {
        self.available_entities.pop().unwrap()
    }

    /* Component */

    pub fn component<G: 'static>(&mut self, entity: usize, component: G) {
        if self.component_pool_exists::<G>() {
            self.assign_component(entity, component);
        } else {
            self.new_component_pool(entity, component);
        }
    }

    pub fn vectorized_component<G: 'static>(&mut self, entity: usize, component: G) {
        if self.component_exists::<Vec<G>>(entity) {
            self.get_component_mut::<Vec<G>>(entity).push(component);
        } else if self.component_pool_exists::<Vec<G>>() {
            self.assign_component(entity, vec![component]);
        } else {
            self.new_component_pool(entity, vec![component]);
        }
    }

    pub fn assign_component<G: 'static>(&mut self, entity: usize, component: G) {
        self.get_mut::<G>().assign_component(entity, component);
    }

    pub fn new_component_pool<G: 'static>(&mut self, entity: usize, component: G) {
        let type_id = TypeId::of::<G>();
        let component_pool = Box::new(ComponentPool::new_with_entity(entity, component));

        self.component_pools.insert(type_id, component_pool);
    }
}

impl<G> ComponentPool<G> {
    pub fn new_with_entity(entity: usize, component: G) -> Self {
        let mut sparse_array = Vec::with_capacity(entity + 1);
        Self::add_entity_to_sparse_array(entity, 0, &mut sparse_array);

        Self {
            num_components: 1,

            sparse_array,
            packed_array: vec![entity],
            component_array: vec![component],
        }
    }

    pub fn assign_component(&mut self, entity: usize, component: G) {
        if self.has_component(entity) {
            self.component_array[self.sparse_array[entity] as usize] = component;
        } else {
            Self::add_entity_to_sparse_array(entity, self.num_components, &mut self.sparse_array);

            self.packed_array.push(entity);
            self.component_array.push(component);
            self.num_components += 1;
        }
    }

    pub fn add_entity_to_sparse_array(entity: usize, value: usize, sparse_array: &mut Vec<i32>) {
        Self::prolong_sparse_array(entity, sparse_array);
        sparse_array[entity] = value as i32;
    }

    pub fn prolong_sparse_array(entity: usize, sparse_array: &mut Vec<i32>) {
        if entity + 1 > sparse_array.len() {
            sparse_array.resize(entity + 1, -1);
        }
    }
}

pub trait EntityConstructor {
    fn create_empty_entity(&mut self);
}

impl<G: 'static> EntityConstructor for ComponentPool<G> {
    fn create_empty_entity(&mut self) {
        self.sparse_array.push(-1);
    }
}

#[cfg(test)]
mod tests {
    use crate::life::{genesis::EntityConstructor, ComponentPool, Scene};

    #[test]
    fn creating_an_entity_increments_an_id() {
        let mut scene = Scene::new();
        let mut entities: Vec<usize> = Vec::new();

        for _i in 0..10 {
            entities.push(scene.entity());
        }

        assert_eq!(vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9], entities);
    }

    #[test]
    fn creating_an_entity_after_having_deleted_one_uses_recycled_id() {
        let mut scene = Scene::new();

        let entity = scene.entity();
        scene.delete(entity);

        assert_eq!(0, scene.entity());
    }

    #[test]
    fn creating_new_component_pool_updates_scene() {
        let mut scene = Scene::new();

        let entity = scene.entity();
        scene.component(entity, 34 as i32);

        assert_eq!(scene.get::<i32>().iter().collect::<Vec<&i32>>(), vec![&34]);
    }

    #[test]
    fn assigning_component_updates_scene() {
        let mut scene = Scene::new();

        let entity1 = scene.entity();
        scene.component(entity1, 34 as i32);

        let entity2 = scene.entity();
        scene.component(entity2, 25 as i32);

        assert_eq!(
            scene.get::<i32>().iter().collect::<Vec<&i32>>(),
            vec![&34, &25]
        );
    }

    #[test]
    fn assigning_already_existing_component_modifies_current_component() {
        let mut scene = Scene::new();
        let entity = scene.entity();

        scene.component(entity, 34 as i32);
        scene.component(entity, 25 as i32);

        assert_eq!(scene.get::<i32>().iter().collect::<Vec<&i32>>(), vec![&25]);
    }

    #[test]
    fn component_pool_creation_works() {
        let pool = ComponentPool::new_with_entity(3, 32);

        assert_eq!(
            pool,
            ComponentPool {
                num_components: 1,

                sparse_array: vec![-1, -1, -1, 0],
                packed_array: vec![3],
                component_array: vec![32],
            },
        );
    }

    #[test]
    fn assigning_component_updates_component_pool() {
        let mut pool = ComponentPool::new_with_entity(3, 32);
        pool.assign_component(6, 28);

        assert_eq!(
            pool,
            ComponentPool {
                num_components: 2,

                sparse_array: vec![-1, -1, -1, 0, -1, -1, 1],
                packed_array: vec![3, 6],
                component_array: vec![32, 28],
            },
        );
    }

    #[test]
    fn assigning_already_existing_component_does_not_add_component() {
        let mut pool = ComponentPool::new_with_entity(0, 32);
        pool.assign_component(0, 28);

        assert_eq!(
            pool,
            ComponentPool {
                num_components: 1,

                sparse_array: vec![0],
                packed_array: vec![0],
                component_array: vec![28],
            },
        );
    }

    #[test]
    fn prolonging_sparse_array_works_as_intended() {
        let mut sparse_array = vec![-1, -1, 0];
        ComponentPool::<i32>::prolong_sparse_array(5, &mut sparse_array);

        assert_eq!(vec![-1, -1, 0, -1, -1, -1], sparse_array,);
    }

    #[test]
    fn prolonging_sparse_array_with_a_smaller_than_length_id_does_nothing() {
        let mut sparse_array = vec![-1, -1, 0];
        ComponentPool::<i32>::prolong_sparse_array(2, &mut sparse_array);

        assert_eq!(vec![-1, -1, 0], sparse_array,);
    }

    #[test]
    fn creating_new_entity_in_component_pool_works_correctly() {
        let mut pool = ComponentPool::new_with_entity(3, 32);
        pool.create_empty_entity();

        assert_eq!(
            pool,
            ComponentPool {
                num_components: 1,

                sparse_array: vec![-1, -1, -1, 0, -1],
                packed_array: vec![3],
                component_array: vec![32],
            },
        );
    }

    #[test]
    fn creating_vectorized_component_encapsulates_it_in_vector() {
        let mut scene = Scene::new();

        let entity = scene.entity();
        scene.vectorized_component(entity, 34 as i32);

        assert_eq!(scene.component_pool_exists::<Vec<i32>>(), true);
    }

    #[test]
    fn adding_to_vectorized_component_pushes_to_vector() {
        let mut scene = Scene::new();

        let entity = scene.entity();
        scene.vectorized_component(entity, 34 as i32);
        scene.vectorized_component(entity, 59 as i32);

        assert_eq!(
            scene.get::<Vec<i32>>().iter().collect::<Vec<&Vec<i32>>>(),
            vec![&vec![34, 59]]
        );
    }

    #[test]
    fn adding_to_second_vectorized_component_pushes_to_vector() {
        let mut scene = Scene::new();

        let entity1 = scene.entity();
        scene.vectorized_component(entity1, 34 as i32);
        scene.vectorized_component(entity1, 59 as i32);

        let entity2 = scene.entity();
        scene.vectorized_component(entity2, 63 as i32);
        scene.vectorized_component(entity2, 16 as i32);

        assert_eq!(
            scene.get::<Vec<i32>>().iter().collect::<Vec<&Vec<i32>>>(),
            vec![&vec![34, 59], &vec![63, 16]]
        );
    }

    #[test]
    fn adding_differently_typed_vectorized_components_does_not_crash() {
        let mut scene = Scene::new();

        let entity1 = scene.entity();
        scene.vectorized_component(entity1, 34 as i32);
        scene.vectorized_component(entity1, 0.59 as f32);
        scene.vectorized_component(entity1, 81 as i32);

        let entity2 = scene.entity();
        scene.vectorized_component(entity2, 63 as u32);
        scene.vectorized_component(entity2, 16 as u32);

        assert_eq!(
            scene.get::<Vec<i32>>().iter().collect::<Vec<&Vec<i32>>>(),
            vec![&vec![34, 81]]
        );

        assert_eq!(
            scene.get::<Vec<f32>>().iter().collect::<Vec<&Vec<f32>>>(),
            vec![&vec![0.59]]
        );

        assert_eq!(
            scene.get::<Vec<u32>>().iter().collect::<Vec<&Vec<u32>>>(),
            vec![&vec![63, 16]]
        );
    }
}