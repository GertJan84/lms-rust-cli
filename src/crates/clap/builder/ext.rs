use crate::crates::clap::util::AnyValueId;
use crate::crates::clap::util::FlatMap;

#[derive(Default, Clone, Debug)]
pub struct Extensions {
    extensions: FlatMap<AnyValueId, BoxedExtension>,
}

impl Extensions {
    #[allow(dead_code)]
    pub fn get<T: Extension>(&self) -> Option<&T> {
        let id = AnyValueId::of::<T>();
        self.extensions.get(&id).map(|e| e.as_ref::<T>())
    }

    #[allow(dead_code)]
    pub fn get_mut<T: Extension>(&mut self) -> Option<&mut T> {
        let id = AnyValueId::of::<T>();
        self.extensions.get_mut(&id).map(|e| e.as_mut::<T>())
    }

    #[allow(dead_code)]
    pub fn get_or_insert_default<T: Extension + Default>(&mut self) -> &mut T {
        let id = AnyValueId::of::<T>();
        self.extensions
            .entry(id)
            .or_insert_with(|| BoxedExtension::new(T::default()))
            .as_mut::<T>()
    }

    #[allow(dead_code)]
    pub fn set<T: Extension + Into<BoxedEntry>>(&mut self, tagged: T) -> bool {
        let BoxedEntry { id, value } = tagged.into();
        self.extensions.insert(id, value).is_some()
    }

    #[allow(dead_code)]
    pub fn remove<T: Extension>(&mut self) -> Option<Box<dyn Extension>> {
        let id = AnyValueId::of::<T>();
        self.extensions.remove(&id).map(BoxedExtension::into_inner)
    }

    pub fn update(&mut self, other: &Self) {
        for (key, value) in other.extensions.iter() {
            self.extensions.insert(*key, value.clone());
        }
    }
}

/// Supports conversion to `Any`. Traits to be extended by `impl_downcast!` must extend `Extension`.
pub trait Extension: std::fmt::Debug + Send + Sync + 'static {
    /// Convert `Box<dyn Trait>` (where `Trait: Extension`) to `Box<dyn Any>`.
    ///
    /// `Box<dyn Any>` can /// then be further `downcast` into
    /// `Box<ConcreteType>` where `ConcreteType` implements `Trait`.
    fn into_any(self: Box<Self>) -> Box<dyn std::any::Any>;
    /// Clone `&Box<dyn Trait>` (where `Trait: Extension`) to `Box<dyn Extension>`.
    ///
    /// `Box<dyn Any>` can /// then be further `downcast` into
    // `Box<ConcreteType>` where `ConcreteType` implements `Trait`.
    fn clone_extension(&self) -> Box<dyn Extension>;
    /// Convert `&Trait` (where `Trait: Extension`) to `&Any`.
    ///
    /// This is needed since Rust cannot /// generate `&Any`'s vtable from
    /// `&Trait`'s.
    fn as_any(&self) -> &dyn std::any::Any;
    /// Convert `&mut Trait` (where `Trait: Extension`) to `&Any`.
    ///
    /// This is needed since Rust cannot /// generate `&mut Any`'s vtable from
    /// `&mut Trait`'s.
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any;
}

impl<T> Extension for T
where
    T: Clone + std::fmt::Debug + Send + Sync + 'static,
{
    fn into_any(self: Box<Self>) -> Box<dyn std::any::Any> {
        self
    }
    fn clone_extension(&self) -> Box<dyn Extension> {
        Box::new(self.clone())
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

impl Clone for Box<dyn Extension> {
    fn clone(&self) -> Self {
        self.as_ref().clone_extension()
    }
}

#[derive(Clone)]
#[repr(transparent)]
struct BoxedExtension(Box<dyn Extension>);

impl BoxedExtension {
    fn new<T: Extension>(inner: T) -> Self {
        Self(Box::new(inner))
    }

    fn into_inner(self) -> Box<dyn Extension> {
        self.0
    }

    fn as_ref<T: Extension>(&self) -> &T {
        self.0.as_ref().as_any().downcast_ref::<T>().unwrap()
    }

    fn as_mut<T: Extension>(&mut self) -> &mut T {
        self.0.as_mut().as_any_mut().downcast_mut::<T>().unwrap()
    }
}

impl std::fmt::Debug for BoxedExtension {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        self.0.fmt(f)
    }
}

#[derive(Clone)]
pub struct BoxedEntry {
    id: AnyValueId,
    value: BoxedExtension,
}

impl BoxedEntry {
    pub fn new(r: impl Extension) -> Self {
        let id = AnyValueId::from(&r);
        let value = BoxedExtension::new(r);
        BoxedEntry { id, value }
    }
}

impl<R: Extension> From<R> for BoxedEntry {
    fn from(inner: R) -> Self {
        BoxedEntry::new(inner)
    }
}
