use iced_wgpu::{pick_list, PickList, Renderer};
use iced_winit::{Element, Row};

#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Component {
    X,
    Y,
    Z,
    W,
}

impl From<Component> for naga::SwizzleComponent {
    fn from(val: Component) -> Self {
        match val {
            Component::X => Self::X,
            Component::Y => Self::Y,
            Component::Z => Self::Z,
            Component::W => Self::W,
        }
    }
}

impl ToString for Component {
    fn to_string(&self) -> String {
        String::from(match self {
            Self::X => "X",
            Self::Y => "Y",
            Self::Z => "Z",
            Self::W => "W",
        })
    }
}

#[derive(Clone, Debug)]
pub enum Message {
    X(Component),
    Y(Component),
    Z(Component),
    W(Component),
}

pub struct State {
    state: [pick_list::State<Component>; 4],
    pattern: [Component; 4],
}

impl Default for State {
    fn default() -> Self {
        Self {
            state: [
                Default::default(),
                Default::default(),
                Default::default(),
                Default::default(),
            ],
            pattern: [Component::X, Component::Y, Component::Z, Component::W],
        }
    }
}

impl State {
    pub fn update(&mut self, message: Message) {
        match message {
            Message::X(component) => self.pattern[0] = component,
            Message::Y(component) => self.pattern[1] = component,
            Message::Z(component) => self.pattern[2] = component,
            Message::W(component) => self.pattern[3] = component,
        }
    }

    pub fn expr(
        &self,
        size: naga::VectorSize,
        vector: naga::Handle<naga::Expression>,
    ) -> naga::Expression {
        naga::Expression::Swizzle {
            size,
            vector,
            pattern: [
                self.pattern[0].into(),
                self.pattern[1].into(),
                self.pattern[2].into(),
                self.pattern[3].into(),
            ],
        }
    }

    pub fn view(&mut self) -> Element<Message, Renderer> {
        fn style<T>(list: PickList<T, Message>) -> PickList<T, Message>
        where
            T: ToString + Eq,
            [T]: ToOwned<Owned = Vec<T>>,
        {
            list.placeholder("")
                .padding([0, 2])
                .text_size(crate::style::FONT_SIZE)
        }

        let opts = &[Component::X, Component::Y, Component::Z, Component::W];
        let [state_x, state_y, state_z, state_w] = &mut self.state;
        let [x, y, z, w] = self.pattern;
        let x = PickList::new(state_x, &opts[..], Some(x), Message::X);
        let y = PickList::new(state_y, &opts[..], Some(y), Message::Y);
        let z = PickList::new(state_z, &opts[..], Some(z), Message::Z);
        let w = PickList::new(state_w, &opts[..], Some(w), Message::W);
        let (x, y, z, w) = (style(x), style(y), style(z), style(w));

        Element::from(Row::new().push(x).push(y).push(z).push(w))
    }
}
