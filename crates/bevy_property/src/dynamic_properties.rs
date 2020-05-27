use crate::{Properties, PropertiesType, Property, PropertyIter, Serializable};
use std::{any::Any, borrow::Cow, collections::HashMap};

pub struct DynamicProperties {
    pub type_name: String,
    pub props: Vec<Box<dyn Property>>,
    pub prop_names: Vec<Cow<'static, str>>,
    pub prop_indices: HashMap<Cow<'static, str>, usize>,
    pub properties_type: PropertiesType,
}

impl DynamicProperties {
    pub fn map() -> Self {
        DynamicProperties {
            type_name: std::any::type_name::<Self>().to_string(),
            props: Default::default(),
            prop_names: Default::default(),
            prop_indices: Default::default(),
            properties_type: PropertiesType::Map,
        }
    }

    pub fn seq() -> Self {
        DynamicProperties {
            type_name: std::any::type_name::<Self>().to_string(),
            props: Default::default(),
            prop_names: Default::default(),
            prop_indices: Default::default(),
            properties_type: PropertiesType::Seq,
        }
    }

    pub fn push(&mut self, prop: Box<dyn Property>, name: Option<&str>) {
        // TODO: validate map / seq operations
        self.props.push(prop);
        if let Some(name) = name {
            let cow_name: Cow<'static, str> = Cow::Owned(name.to_string()); // moo
            self.prop_names.push(cow_name.clone());
            self.prop_indices.insert(cow_name, self.props.len() - 1);
        }
    }
    pub fn set<T: Property>(&mut self, name: &str, prop: T) {
        // TODO: validate map / seq operations
        if let Some(index) = self.prop_indices.get(name) {
            self.props[*index] = Box::new(prop);
        } else {
            self.push(Box::new(prop), Some(name));
        }
    }
    pub fn set_box(&mut self, name: &str, prop: Box<dyn Property>) {
        // TODO: validate map / seq operations
        if let Some(index) = self.prop_indices.get(name) {
            self.props[*index] = prop;
        } else {
            self.push(prop, Some(name));
        }
    }
}

impl Properties for DynamicProperties {
    #[inline]
    fn prop(&self, name: &str) -> Option<&dyn Property> {
        if let Some(index) = self.prop_indices.get(name) {
            Some(&*self.props[*index])
        } else {
            None
        }
    }

    #[inline]
    fn prop_mut(&mut self, name: &str) -> Option<&mut dyn Property> {
        if let Some(index) = self.prop_indices.get(name) {
            Some(&mut *self.props[*index])
        } else {
            None
        }
    }

    #[inline]
    fn prop_with_index(&self, index: usize) -> Option<&dyn Property> {
        self.props.get(index).map(|prop| &**prop)
    }

    #[inline]
    fn prop_with_index_mut(&mut self, index: usize) -> Option<&mut dyn Property> {
        self.props.get_mut(index).map(|prop| &mut **prop)
    }

    #[inline]
    fn prop_name(&self, index: usize) -> Option<&str> {
        match self.properties_type {
            PropertiesType::Seq => None,
            PropertiesType::Map => self.prop_names.get(index).map(|name| name.as_ref()),
        }
    }

    #[inline]
    fn prop_len(&self) -> usize {
        self.props.len()
    }

    fn iter_props(&self) -> PropertyIter {
        PropertyIter {
            props: self,
            index: 0,
        }
    }

    #[inline]
    fn properties_type(&self) -> PropertiesType {
        self.properties_type
    }
}

impl Property for DynamicProperties {
    #[inline]
    fn type_name(&self) -> &str {
        &self.type_name
    }

    #[inline]
    fn any(&self) -> &dyn Any {
        self
    }
    #[inline]
    fn any_mut(&mut self) -> &mut dyn Any {
        self
    }
    #[inline]
    fn clone_prop(&self) -> Box<dyn Property> {
        Box::new(self.to_dynamic())
    }
    #[inline]
    fn set(&mut self, value: &dyn Property) {
        if let Some(properties) = value.as_properties() {
            *self = properties.to_dynamic();
        } else {
            panic!("attempted to apply non-Properties type to Properties type");
        }
    }

    #[inline]
    fn apply(&mut self, value: &dyn Property) {
        if let Some(properties) = value.as_properties() {
            if properties.properties_type() != self.properties_type {
                panic!(
                    "Properties type mismatch. This type is {:?} but the applied type is {:?}",
                    self.properties_type,
                    properties.properties_type()
                );
            }
            match self.properties_type {
                PropertiesType::Map => {
                    for (i, prop) in properties.iter_props().enumerate() {
                        let name = properties.prop_name(i).unwrap();
                        self.prop_mut(name).map(|p| p.apply(prop));
                    }
                }
                PropertiesType::Seq => {
                    for (i, prop) in properties.iter_props().enumerate() {
                        self.prop_with_index_mut(i).map(|p| p.apply(prop));
                    }
                }
            }
        } else {
            panic!("attempted to apply non-Properties type to Properties type");
        }
    }

    fn as_properties(&self) -> Option<&dyn Properties> {
        Some(self)
    }

    fn is_sequence(&self) -> bool {
        self.properties_type == PropertiesType::Seq
    }

    fn serializable(&self) -> Serializable {
        Serializable::Borrowed(self)
    }
}
