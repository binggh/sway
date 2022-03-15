use crate::priv_prelude::*;

#[derive(Clone, Debug)]
pub struct ItemStruct {
    pub visibility: Option<PubToken>,
    pub struct_token: StructToken,
    pub name: Ident,
    pub generics: Option<GenericParams>,
    pub fields: Braces<Punctuated<TypeField, CommaToken>>,
}

impl ItemStruct {
    pub fn span(&self) -> Span {
        let start = match &self.visibility {
            Some(pub_token) => pub_token.span(),
            None => self.struct_token.span(),
        };
        let end = self.fields.span();
        Span::join(start.clone(), end)
    }
}

impl Parse for ItemStruct {
    fn parse(parser: &mut Parser) -> ParseResult<ItemStruct> {
        let visibility = parser.take();
        let struct_token = parser.parse()?;
        let name = parser.parse()?;
        let generics = if parser.peek::<LessThanToken>().is_some() {
            Some(parser.parse()?)
        } else {
            None
        };
        let fields = parser.parse()?;
        Ok(ItemStruct { visibility, struct_token, name, generics, fields })
    }
}

