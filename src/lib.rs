use egui::{Button, Grid, Response, ScrollArea, Ui, Widget, WidgetText};
pub use egui_struct_macros::*;

macro_rules! generate_show {
    ($top_name:ident, $collapsing_name:ident, $primitive_name:ident, $childs_name:ident, $typ:ty, $config:ident, $COLUMN_COUNT:ident, $SIMPLE:ident, $has_childs_imut:ident, $has_primitive:ident) => {
        type $config: Default;
        const $COLUMN_COUNT: usize = 2;
        const $SIMPLE: bool = true;
        fn $has_childs_imut(&self) -> bool {
            false
        }
        fn $has_primitive(&self) -> bool {
            !self.$has_childs_imut()
        }

        fn $top_name(
            self: $typ,
            ui: &mut Ui,
            label: impl Into<WidgetText> + Clone,
            reset2: Option<&Self>,
        ) -> Response {
            ScrollArea::vertical()
                .auto_shrink([false; 2])
                .show(ui, |ui| {
                    Grid::new(ui.next_auto_id())
                        .striped(true)
                        .num_columns(Self::$COLUMN_COUNT)
                        .show(ui, |ui| {
                            self.$collapsing_name(ui, label, "", -1, Default::default(), reset2)
                        })
                        .inner
                })
                .inner
        }

        fn $collapsing_name(
            self: $typ,
            ui: &mut Ui,
            label: impl Into<WidgetText> + Clone,
            hint: impl Into<WidgetText> + Clone,
            indent_level: isize,
            config: Self::$config,
            _reset2: Option<&Self>,
        ) -> Response {
            let mut uncollapsed = true;
            let has_childs_imut = self.$has_childs_imut();
            ui.horizontal(|ui| {
                if indent_level >= 0 {
                    for _ in 0..indent_level {
                        ui.separator();
                    }
                    if has_childs_imut {
                        let id = ui.make_persistent_id((
                            self as *const Self as *const usize as usize,
                            indent_level,
                        ));
                        uncollapsed = ui.data_mut(|d| d.get_temp_mut_or(id, true).clone());
                        let icon = if uncollapsed { "⏷" } else { "⏵" };
                        if Button::new(icon).frame(false).small().ui(ui).clicked() {
                            ui.data_mut(|d| d.insert_temp(id, !uncollapsed));
                        }
                    }
                }
                let mut lab = ui.label(label.into());
                let hint = hint.into();
                if !hint.is_empty() {
                    lab = lab.on_hover_text(hint);
                }
                lab
            });

            let mut ret = ui
                .horizontal(|ui| {
                    #[allow(unused_mut)]
                    let mut ret = self.$primitive_name(ui, config);
                    macro_rules! reset {
                        (show_collapsing_imut) => {
                            ret
                        };
                        (show_collapsing) => {
                            if let Some(reset2) = _reset2 {
                                if !reset2.eguis_eq(self) {
                                    let mut r = ui.button("⟲");
                                    if r.clicked() {
                                        self.eguis_clone(reset2);
                                        r.mark_changed();
                                    }
                                    ret |= r;
                                }
                            }
                            ret
                        };
                    }
                    reset! {$collapsing_name}
                })
                .inner;
            ui.end_row();

            if has_childs_imut && uncollapsed {
                ret = self.$childs_name(ui, indent_level + 1, ret, _reset2);
            }
            ret
        }
        fn $primitive_name(self: $typ, ui: &mut Ui, _config: Self::$config) -> Response {
            ui.label("")
        }
        fn $childs_name(
            self: $typ,
            _ui: &mut Ui,
            _indent_level: isize,
            _response: Response,
            _reset2: Option<&Self>,
        ) -> Response {
            unreachable!()
        }
    };
}
pub trait EguiStructClone {
    fn eguis_clone(&mut self, source: &Self);
}
pub trait EguiStructEq {
    fn eguis_eq(&self, _rhs: &Self) -> bool {
        //default implementation can be used if reset button is not required
        true
    }
}
#[macro_export]
macro_rules! impl_eclone {
    ([$($generics:tt)*], $type:ty) => {
        impl<$($generics)*> EguiStructClone for $type {
            fn eguis_clone(&mut self, source: &Self) {
                self.clone_from(source);
            }
        }
    };
}
#[macro_export]
macro_rules! impl_eeq {
    ([$($generics:tt)*], $type:ty) => {
        impl<$($generics)*> EguiStructEq for $type {
            fn eguis_eq(&self, rhs: &Self) -> bool {
                self == rhs
            }
        }
    };
}
#[macro_export]
macro_rules! impl_eeqclone {
    ([$($generics:tt)*], $type:ty) => {
        impl_eeq!{[$($generics)*], $type}
        impl_eclone!{[$($generics)*], $type}
    };
    ($type:ty) => {impl_eeqclone!{[],$type}}
}

pub trait EguiStruct: EguiStructClone + EguiStructEq {
    generate_show! { show_top, show_collapsing, show_primitive, show_childs, &mut Self, ConfigType, COLUMN_COUNT, SIMPLE, has_childs, has_primitive }
}
pub trait EguiStructImut {
    generate_show! { show_top_imut, show_collapsing_imut, show_primitive_imut, show_childs_imut, &Self, ConfigTypeImut, COLUMN_COUNT_IMUT, SIMPLE_IMUT, has_childs_imut, has_primitive_imut }
}

///Config structure for mutable view of Numerics
#[derive(Default)]
pub enum ConfigNum<T> {
    ///Default: DragValue (without limits)
    #[default]
    NumDefault,

    ///DragValue(min, max)
    DragValue(T, T),

    ///Slider(min, max)
    Slider(T, T),
}
macro_rules! impl_num_primitives {
    ($($typ:ty)*) => {
        $(
            impl EguiStruct for $typ {
                type ConfigType = ConfigNum<$typ>;
                fn show_primitive(&mut self, ui: &mut Ui, config: Self::ConfigType) -> Response {
                    match config{
                        Self::ConfigType::NumDefault        =>  egui::DragValue::new(self).ui(ui),
                        Self::ConfigType::DragValue(min,max)=>  egui::DragValue::new(self).clamp_range(min..=max).ui(ui),
                        Self::ConfigType::Slider(min,max)   =>  egui::Slider::new(self, min..=max).ui(ui),
                    }
                }
            }
            impl EguiStructImut for $typ {
                type ConfigTypeImut = ConfigStrImut;
                fn show_primitive_imut(&self, ui: &mut Ui, config: Self::ConfigTypeImut) -> Response {
                    self.to_string().as_str().show_primitive_imut(ui, config)
                }
            }
            impl_eeqclone!{$typ}
        )*
    };
}

impl_num_primitives!(i8 i16 i32 i64 u8 u16 u32 u64 usize isize f32 f64);

impl EguiStruct for bool {
    type ConfigType = ();
    fn show_primitive(&mut self, ui: &mut Ui, _config: Self::ConfigType) -> Response {
        egui::Checkbox::without_text(self).ui(ui)
    }
}
impl EguiStructImut for bool {
    type ConfigTypeImut = ();
    fn show_primitive_imut(&self, ui: &mut Ui, _config: Self::ConfigTypeImut) -> Response {
        ui.add_enabled(false, egui::Checkbox::without_text(&mut self.clone()))
    }
}
impl_eeqclone! {bool}
/////////////////////////////////////////////////////////
///Config structure for mutable view of String
#[derive(Default)]
pub enum ConfigStr {
    ///Default: single line `egui::TextEdit`
    #[default]
    SingleLine,

    ///multi line `egui::TextEdit`
    MultiLine,
}

///Config structure for immutable view of many simple types like str, String & numerics
#[derive(Default)]
pub enum ConfigStrImut {
    ///`egui::Label`
    NonSelectable,

    ///Default: imutable `egui::TextEdit`
    #[default]
    Selectable,
}

impl EguiStruct for String {
    type ConfigType = ConfigStr;
    fn show_primitive(&mut self, ui: &mut Ui, config: Self::ConfigType) -> Response {
        match config {
            ConfigStr::SingleLine => ui.text_edit_singleline(self),
            ConfigStr::MultiLine => ui.text_edit_multiline(self),
        }
    }
}
impl EguiStructImut for String {
    type ConfigTypeImut = ConfigStrImut;
    fn show_primitive_imut(&self, ui: &mut Ui, config: Self::ConfigTypeImut) -> Response {
        self.as_str().show_primitive_imut(ui, config)
    }
}
impl_eeqclone! {String}

impl EguiStructImut for str {
    type ConfigTypeImut = ConfigStrImut;
    fn show_primitive_imut(mut self: &Self, ui: &mut Ui, config: Self::ConfigTypeImut) -> Response {
        match config {
            ConfigStrImut::NonSelectable => ui.label(self),
            ConfigStrImut::Selectable => ui.text_edit_singleline(&mut self),
        }
    }
}

/////////////////////////////////////////////////////////
impl<T: EguiStructImut + Default> EguiStructImut for Option<T> {
    const SIMPLE_IMUT: bool = false;
    type ConfigTypeImut = ();
    fn has_childs_imut(&self) -> bool {
        !T::SIMPLE_IMUT && self.is_some()
    }
    fn has_primitive_imut(&self) -> bool {
        true
    }
    fn show_primitive_imut(&self, ui: &mut Ui, _config: Self::ConfigTypeImut) -> Response {
        ui.horizontal(|ui| {
            let mut ret = self.is_some().show_primitive_imut(ui, ());
            match (T::SIMPLE_IMUT, self) {
                (true, Some(value)) => ret |= value.show_primitive_imut(ui, Default::default()),
                (true, None) => (),
                (false, _) => (),
            }
            ret
        })
        .inner
    }
    fn show_childs_imut(
        &self,
        ui: &mut Ui,
        indent_level: isize,
        mut response: Response,
        _reset2: Option<&Self>,
    ) -> Response {
        if let Some(inner) = self {
            if inner.has_primitive_imut() {
                response |= inner.show_collapsing_imut(
                    ui,
                    "[0]",
                    "",
                    indent_level,
                    Default::default(),
                    None,
                );
            } else {
                response |= inner.show_childs_imut(ui, indent_level, response.clone(), None)
            }
        }
        response
    }
}
impl<T: EguiStruct + Default> EguiStruct for Option<T> {
    const SIMPLE: bool = false;
    type ConfigType = ();
    fn has_childs(&self) -> bool {
        !T::SIMPLE && self.is_some()
    }
    fn has_primitive(&self) -> bool {
        true
    }
    fn show_primitive(&mut self, ui: &mut Ui, _config: Self::ConfigType) -> Response {
        ui.horizontal(|ui| {
            let mut checked = self.is_some();
            let mut ret = checked.show_primitive(ui, ());

            match (checked, T::SIMPLE, self.as_mut()) {
                (true, true, Some(value)) => ret |= value.show_primitive(ui, Default::default()),
                (true, false, Some(_)) => (), //if inner is not simple, it will be shown below
                (true, _, None) => *self = Some(T::default()),
                (false, _, _) => *self = None,
            }
            ret
        })
        .inner
    }
    fn show_childs(
        &mut self,
        ui: &mut Ui,
        indent_level: isize,
        mut response: Response,
        reset2: Option<&Self>,
    ) -> Response {
        if let Some(inner) = self {
            if inner.has_primitive() {
                response |= inner.show_collapsing(
                    ui,
                    "[0]",
                    "",
                    indent_level,
                    Default::default(),
                    reset2.unwrap_or(&None).as_ref(),
                );
            } else {
                response |= inner.show_childs(
                    ui,
                    indent_level,
                    response.clone(),
                    reset2.unwrap_or(&None).as_ref(),
                )
            }
        }
        response
    }
}
impl<T: EguiStructClone + Default> EguiStructClone for Option<T> {
    fn eguis_clone(&mut self, source: &Self) {
        if let Some(source) = source {
            if let Some(s) = self {
                s.eguis_clone(source);
            } else {
                let mut v: T = Default::default();
                v.eguis_clone(source);
                *self = Some(v);
            }
        } else {
            *self = None;
        }
    }
}
impl<T: EguiStructEq> EguiStructEq for Option<T> {
    fn eguis_eq(&self, rhs: &Self) -> bool {
        if let Some(s) = self {
            if let Some(r) = rhs {
                s.eguis_eq(r)
            } else {
                false
            }
        } else {
            false
        }
    }
}
///////////////////////////////////////////////////
macro_rules! impl_vec {
    ($Self:ty, $typ:ty, $iter:ident, $collapsing_name:ident, $childs_name:ident,$trait:ident, $SIMPLE:ident, $ConfigType:ident, $has_childs_imut:ident, $has_primitive:ident) => {
        impl<T: $trait> $trait for $typ{
            const $SIMPLE: bool = false;
            type $ConfigType = ();
            fn $has_childs_imut(&self) -> bool {
                !self.is_empty()
            }
            fn $has_primitive(&self) -> bool {
                false
            }
            fn $childs_name(
                self: $Self,
                ui: &mut Ui,
                indent_level: isize,
                mut response: Response,
                _reset2: Option<&Self>,
            ) -> Response {
                self.$iter().enumerate().for_each(|(idx, x)| {
                    response |= x.$collapsing_name(ui, idx.to_string(), "", indent_level, Default::default(), None)
                });
                response
            }
        }
    };
    (IMUT, $($typ:ty)*) => { $(impl_vec! {&Self, $typ, iter, show_collapsing_imut, show_childs_imut, EguiStructImut, SIMPLE_IMUT, ConfigTypeImut, has_childs_imut, has_primitive_imut})* };
    ($($typ:ty)*) => {
        $(
            impl_vec! {IMUT, $typ}
            impl_vec! {&mut Self, $typ, iter_mut, show_collapsing, show_childs, EguiStruct, SIMPLE, ConfigType, has_childs, has_primitive}

            impl<T: EguiStructClone> EguiStructClone for $typ {
                fn eguis_clone(&mut self, source: &Self) {
                    //TODO update this if vector length can change
                    self.iter_mut().zip(source.iter()).for_each(|(s,r)|s.eguis_clone(r))
                }
            }
            impl<T: EguiStructEq> EguiStructEq for $typ  {
                fn eguis_eq(&self, rhs: &Self) -> bool {
                    let mut ret = self.len()==rhs.len();
                    self.iter().zip(rhs.iter()).for_each(|(s,r)|ret &= s.eguis_eq(r));
                    ret
                }
            }
        )*
    };
}

impl_vec! {[T] Vec<T>}
impl_vec! {IMUT, std::collections::HashSet<T> }
#[cfg(feature = "indexmap")]
impl_vec! {IMUT, indexmap::IndexSet<T> }

/////////////////////////////////////////////////
macro_rules! impl_map {
    ($Self:ty, $typ:ty, [$( $Qbound:path),*], $iter:ident, $collapsing_name:ident, $childs_name:ident,$trait:ident, $SIMPLE:ident, $ConfigType:ident, $has_childs_imut:ident, $has_primitive:ident) => {
        impl<Q: ToString $(+ $Qbound)*, V: $trait> $trait for $typ{
            const $SIMPLE: bool = false;
            type $ConfigType = ();
            fn $has_childs_imut(&self) -> bool {
                !self.is_empty()
            }
            fn $has_primitive(&self) -> bool {
                false
            }
            fn $childs_name(
                self: $Self,
                ui: &mut Ui,
                indent_level: isize,
                mut response: Response,
                _reset2: Option<&Self>,
            ) -> Response {
                self.$iter().for_each(|(q, v)| {
                    response |= v.$collapsing_name(
                        ui,
                        q.to_string(),
                        "",
                        indent_level,
                        Default::default(),
                        None,
                    )
                });
                response
            }
        }
    };
    ($typ:ty) => {
        impl_map! {&Self, $typ, [], iter, show_collapsing_imut, show_childs_imut, EguiStructImut, SIMPLE_IMUT, ConfigTypeImut, has_childs_imut, has_primitive_imut}
        impl_map! {&mut Self, $typ, [Eq, std::hash::Hash], iter_mut, show_collapsing, show_childs, EguiStruct, SIMPLE, ConfigType, has_childs, has_primitive}

        impl<Q: ToString + Eq + std::hash::Hash, V: EguiStructClone> EguiStructClone for $typ {
            fn eguis_clone(&mut self, source: &Self) {
                //this is very simplified implementation, that asummes that lenghts & keys are the same
                self.iter_mut().for_each(|(q, v)| {
                    if let Some(r) = source.get(q) {
                        v.eguis_clone(r)
                    }
                })
            }
        }
        impl<Q: ToString + Eq + std::hash::Hash, V: EguiStructEq> EguiStructEq for $typ {
            fn eguis_eq(&self, rhs: &Self) -> bool {
                let mut ret = self.len() == rhs.len();
                self.iter().for_each(|(q, v)| {
                    if let Some(r) = rhs.get(q) {
                        ret &= v.eguis_eq(r)
                    } else {
                        ret = false
                    }
                });
                ret
            }
        }
    };
}

impl_map! { std::collections::HashMap<Q,V> }
#[cfg(feature = "indexmap")]
impl_map! { indexmap::IndexMap<Q,V> }
///////////////////////////////////////////////////////
macro_rules! impl_large_numerics {
    ($($t:ty)*) => ($(
        impl EguiStructImut for $t {
            type ConfigTypeImut = ();
            fn show_primitive_imut(&self, ui: &mut Ui, _config: Self::ConfigTypeImut) -> Response {
                ui.label(self.to_string())
            }
        }
        impl EguiStruct for $t {
            type ConfigType = ();
            fn show_primitive(&mut self, ui: &mut Ui, _config: Self::ConfigType)-> Response  {
                let mut text = self.to_string();
                let ret=ui.text_edit_singleline(&mut text);
                if let Ok(value) = text.parse() {
                    *self = value;
                }
                ret
            }
        }
        impl_eeqclone!{$t}
    )*)
}
impl_large_numerics!(i128 u128);
