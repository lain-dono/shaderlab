use super::{GenError, Message, Node, NodeId, Type};
use iced_native::{widget::pick_list, Element, Length as IcedLength};
use iced_wgpu::Renderer;

#[derive(Debug, Clone)]
struct ChangeType(Type);

macro_rules! custom {

    ($name:ident :: $fn:ident ($($arg:ident),+) -> $ret:ident ($format:literal, $($arg_expr:expr),+)) => {
        #[derive(Clone, Debug)]
        pub struct $name {
            ty_select: pick_list::State<Type>,
            selected: Option<Type>
        }

        impl Default for $name {
            fn default() -> Self {
                Self {
                    ty_select: pick_list::State::default(),
                    selected: Some(Type::V4F),
                }
            }
        }

        impl $name {
            pub fn boxed() -> Box<dyn Node> {
                Box::new(Self::default())
            }
        }

        impl Node for $name {
            fn label(&self) -> &str {
                stringify!($fn)
            }

            fn width(&self) -> u16 {
                80
            }

            fn inputs(&self) -> &[&str] {
                &[$(stringify!($arg)),+]
            }

            fn outputs(&self) -> &[&str] {
                &[stringify!($ret)]
            }

            fn generate(
                &self,
                inputs: &[Option<String>],
                outputs: &[String],
            ) -> Result<String, GenError> {
                if let ([$(Some($arg)),+], [$ret]) = (inputs, outputs) {
                    Ok(format!($format, $($arg_expr),+))
                } else {
                    Err(GenError)
                }
            }

            fn update(&mut self, _node: NodeId, message: Message) {
                let message  = super::downcast_message::<ChangeType>(message).unwrap();
                self.selected = Some(message.0);
            }

            fn view(&mut self, _node: NodeId) -> Element<Message, Renderer> {
                let variants = &[Type::F32, Type::V2F, Type::V3F, Type::V4F][..];
                pick_list::PickList::new(&mut self.ty_select, variants, self.selected, move |x| super::upcast_message(ChangeType(x)))
                    .padding([0, 4])
                    .text_size(crate::style::FONT_SIZE)
                    .width(IcedLength::Fill)
                    .into()
            }
        }
    };
}

macro_rules! math_fn {
    ($name:ident :: $fn:ident($a:ident))
    => { custom!($name::$fn($a) -> out ("let {} = {}({});", out, stringify!($fn), $a)); };
    ($name:ident :: $fn:ident($a:ident, $b:ident))
    => { custom!($name::$fn($a, $b) -> out ("let {} = {}({}, {});", out, stringify!($fn), $a, $b)); };
}

macro_rules! math_op {
    ($name:ident :: $fn:ident($a:ident $op:tt $b:ident))
    => { custom!($name::$fn($a, $b) -> out ("let {} = {} {} {};", out, $a, stringify!($op), $b)); }
}

// Advanced

//math_fn!(Abs abs(a));
math_fn!(Abs::abs(a));
math_fn!(Exp::exp(a));
math_fn!(Exp2::exp2(a));
math_fn!(Length::length(a));
math_fn!(Log::log(a));
math_op!(Rem::rem(a % b));
// negate?
math_fn!(Normalize::normalize(a));
// posterize
// recip
// recip_sqrt/invsqrt

// Basic
//custom!(Add::add(a, b) -> out ("let {} = {} {} {};", out, a, "+", b));
math_op!(Add::add(a + b));
math_op!(Sub::sub(a - b));
math_op!(Mul::mul(a * b));
math_op!(Div::div(a / b));
math_fn!(Pow::pow(a, b));
math_fn!(Sqrt::sqrt(a));

// Derivative
math_fn!(DpDx::dpdx(a));
math_fn!(DpDy::dpdy(a));
//math_fn!(DPDXY::dpdxy(a));

// Trigonometry

math_fn!(Acos::acos(a));
math_fn!(Asin::asin(a));
math_fn!(Atan::atan(a));
math_fn!(Atan2::atan2(a, b));
math_fn!(Cos::cos(a));
math_fn!(Sin::sin(a));
math_fn!(Tan::tan(a));
math_fn!(Cosh::cosh(a));
math_fn!(Sinh::sinh(a));
math_fn!(Tanh::tanh(a));

custom!(RadiansToDegres::to_degres(a) -> out ("let {} = {} * 57.29578;", out, a));
custom!(DegresToRadians::to_radians(a) -> out ("let {} = {} * 0.017453292;", out, a));

// Rounding
math_fn!(Ceil::ceil(a));
math_fn!(Floor::floor(a));
math_fn!(Round::round(a));
math_fn!(Sign::sign(a));
math_fn!(Trunc::trunc(a));
math_fn!(Step::step(a, b));
