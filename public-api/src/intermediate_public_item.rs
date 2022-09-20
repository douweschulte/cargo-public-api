use crate::render;
use std::rc::Rc;

use rustdoc_types::{Id, Item, Type};

use crate::tokens::Token;

/// This struct represents one public item of a crate, but in intermediate form.
/// It wraps a single [Item] but adds additional calculated values to make it
/// easier to work with. Later, one [`Self`] will be converted to exactly one
/// [`crate::PublicItem`].
#[derive(Clone, Debug)]
pub struct IntermediatePublicItem<'a> {
    /// The item we are effectively wrapping.
    pub item: &'a Item,

    /// The name of the item. Normally this is [Item::name]. But in the case of
    /// renamed imports (`pub use other::item as foo;`) it is the new name.
    pub name: String,

    /// Sometimes an item references other items via ID that we do not include
    /// in the output. Currently this only happens for tuple structs (both
    /// regular and enum variants). The reason we can get away with that is
    /// because the tuple type already contains the full info, so also listing
    /// fields as individual items is just noise.
    ///
    /// To be more concrete: It is sufficient to list the public API of `pub
    /// struct Foo(bool, u32)` as
    /// ```txt
    /// pub struct root::Foo(bool, u32)
    /// ```
    /// because listing it as
    /// ```txt
    /// pub struct root::Foo(bool, u32)
    /// pub struct field root::Foo::0: bool
    /// pub struct field root::Foo::1: u32
    /// ```
    /// does not convey any more information.
    ///
    /// This special case requires us to pre-resolve the IDs however, and that
    /// is what this field is for.
    pub pre_resolved_fields: Vec<Option<&'a Type>>,

    /// The parent item. If [Self::item] is e.g. an enum variant, then the
    /// parent is an enum. We follow the chain of parents to be able to know the
    /// correct path to an item in the output.
    parent: Option<Rc<IntermediatePublicItem<'a>>>,
}

impl<'a> IntermediatePublicItem<'a> {
    #[must_use]
    pub fn new(
        item: &'a Item,
        name: String,
        pre_resolved_fields: Vec<Option<&'a Type>>,
        parent: Option<Rc<IntermediatePublicItem<'a>>>,
    ) -> Self {
        Self {
            item,
            name,
            pre_resolved_fields,
            parent,
        }
    }

    #[must_use]
    pub fn path(&'a self) -> Vec<Rc<IntermediatePublicItem<'a>>> {
        let mut path = vec![];

        let rc_self = Rc::new(self.clone());

        path.insert(0, rc_self.clone());

        let mut current_item = rc_self.clone();
        while let Some(parent) = current_item.parent.clone() {
            path.insert(0, parent.clone());
            current_item = parent.clone();
        }

        path
    }

    #[must_use]
    pub fn path_contains_id(&self, id: &'a Id) -> bool {
        self.path().iter().any(|m| m.item.id == *id)
    }

    pub fn render_token_stream(&self) -> Vec<Token> {
        render::token_stream(self)
    }
}
