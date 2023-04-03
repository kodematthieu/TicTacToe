use std::f64::consts::PI;
use std::hint::unreachable_unchecked;
use std::ops::Deref;
use std::time::Duration;

use defaults::Defaults;
use druid::kurbo::{Line, RoundedRect, Arc};
use druid::piet::{StrokeStyle, LineJoin, LineCap, StrokeDash};
use druid::{Widget, Data, Color, RenderContext, Point, Event, WidgetPod, MouseEvent, MouseButton, TimerToken};
use keyframe::{ease, EasingFunction};
use keyframe::functions::{EaseInCubic, EaseInOutQuart};

use crate::engine::{State, TicTacToe};

#[derive(Clone, Data, Defaults)]
pub struct AppState {
    #[def = "TicTacToe::new(State::N)"]
    game: TicTacToe,
    anim: f64
}

struct Animate<T: Sized, E: EasingFunction, const D: u64> {
    time: f64,
    data: T,
    ease: E,
    value: f64
}
impl<T: Sized, E: EasingFunction, const D: u64> Animate<T, E, D> {
    #[inline]
    fn new(data: T, ease: E) -> Self {
        Self {
            data,
            ease,
            value: 0.0,
            time: 0.0
        }
    }
    fn anim_frame(&mut self, t: u64) {
        self.time += t as f64 * 1e-6 / D as f64;
        self.value = ease::<f64, f64, E>(&self.ease, 0.0, 1.0, self.time);
    }
    #[inline]
    fn data(&self) -> &T {
        &self.data
    }
    #[inline]
    fn data_mut(&mut self) -> &mut T {
        &mut self.data
    }
    #[inline]
    fn starting(&self) -> bool {
        self.value <= 0.0 && self.time == 0.0
    }
    #[inline]
    fn finished(&self) -> bool {
        self.value >= 1.0
    }
}
impl<T: Default + Sized, E: Default + EasingFunction, const D: u64> Default for Animate<T, E, D> {
    fn default() -> Self {
        Self {
            time: 0.0,
            data: T::default(),
            ease: E::default(),
            value: 0.0
        }
    }
}
impl<T: Sized, E: EasingFunction, const D: u64> Deref for Animate<T, E, D> {
    type Target = f64;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

struct ReversibleAnimate<T: Sized, E: EasingFunction, const D: u64> {
    time: f64,
    data: T,
    ease: E,
    value: f64,
    reverse: bool
}
impl<T: Sized, E: EasingFunction, const D: u64> ReversibleAnimate<T, E, D> {
    #[inline]
    fn new(data: T, ease: E) -> Self {
        Self {
            data,
            ease,
            value: 0.0,
            time: 0.0,
            reverse: false
        }
    }
    fn is_reverse(&self) -> bool {
        self.reverse
    }
    fn reverse(&mut self) {
        self.reverse = !self.reverse
    }
    fn anim_frame(&mut self, t: u64) {
        if self.reverse {
            self.time -= t as f64 * 1e-6 / D as f64;
            self.value = ease::<f64, f64, E>(&self.ease, 1.0, 0.0, 1.0 - self.time);
        } else {
            self.time += t as f64 * 1e-6 / D as f64;
            self.value = ease::<f64, f64, E>(&self.ease, 0.0, 1.0, self.time);
        }
    }
    #[inline]
    fn data(&self) -> &T {
        &self.data
    }
    #[inline]
    fn data_mut(&mut self) -> &mut T {
        &mut self.data
    }
    #[inline]
    fn starting(&self) -> bool {
        if self.reverse {
            self.value >= 1.0 && self.time >= 1.0
        } else {
            self.value <= 0.0 && self.time <= 0.0
        }
    }
    #[inline]
    fn finished(&self) -> bool {
        if self.reverse {
            self.value <= 0.0 && self.time <= 0.0
        } else {
            self.value >= 1.0 && self.time >= 1.0
        }
    }
}
impl<T: Default + Sized, E: Default + EasingFunction, const D: u64> Default for ReversibleAnimate<T, E, D> {
    fn default() -> Self {
        Self::new(Default::default(), Default::default())
    }
}
impl<T: Sized, E: EasingFunction, const D: u64> Deref for ReversibleAnimate<T, E, D> {
    type Target = f64;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

pub struct GridCell {
    idx: u8,
    state: ReversibleAnimate<bool, EaseInOutQuart, 500>,
    hover: ReversibleAnimate<(), EaseInOutQuart, 100>
}
impl GridCell {
    fn new(idx: u8) -> Self {
        let mut state = ReversibleAnimate::default();
        state.reverse();
        let mut hover = ReversibleAnimate::default();
        hover.reverse();
        Self {
            idx,
            state,
            hover
        }
    }
}
impl Widget<AppState> for GridCell {
    fn event(&mut self, ctx: &mut druid::EventCtx, event: &Event, data: &mut AppState, _: &druid::Env) {
        let mut hot_change = false;
        match event {
            &Event::AnimFrame(t) => {
                ctx.request_paint();
                if !self.state.finished() {
                    self.state.anim_frame(t);
                    ctx.request_anim_frame();
                }
                if !self.hover.finished() {
                    self.hover.anim_frame(t);
                    ctx.request_anim_frame();
                }
            },
            &Event::MouseDown(MouseEvent { button: MouseButton::Left, .. }) if data.game.done().is_none() => {
                let state = data.game.state();
                if data.game.set(self.idx as _) {
                    *self.state.data_mut() = match state {
                        State::X => false,
                        State::O => true,
                        _ => unsafe {unreachable_unchecked()}
                    };
                    self.state.reverse();
                    ctx.request_anim_frame();
                    ctx.request_update();
                    hot_change = true;
                }
            },
            &Event::MouseMove(_) => {
                hot_change = true;
            }
            _ => ()
        }
        
        match (ctx.is_hot(), self.hover.is_reverse()) {
            (true, false) if !self.state.is_reverse() || data.game.done().is_some() => {
                self.hover.reverse();
                ctx.request_anim_frame();
            },
            (true,  true) if !self.state.is_reverse() || data.game.done().is_some() => (),
            (x, y) if x == y => {
                self.hover.reverse();
                ctx.request_anim_frame();
            },
            _ => ()
        }
    }

    fn lifecycle(&mut self, ctx: &mut druid::LifeCycleCtx, _: &druid::LifeCycle, data: &AppState, _: &druid::Env) {}

    fn update(&mut self, ctx: &mut druid::UpdateCtx, old_data: &AppState, data: &AppState, _: &druid::Env) {
        let state = data.game.get(self.idx as _);
        match state {
            State::N if old_data.game.get(self.idx as _) != State::N => {
                self.state.reverse();
                ctx.request_anim_frame();
            },
            _ => ()
        }
    }

    fn layout(&mut self, _: &mut druid::LayoutCtx, bc: &druid::BoxConstraints, _: &AppState, _: &druid::Env) -> druid::Size {
        bc.constrain_aspect_ratio(1.0, bc.max().width / 3.0)
    }

    fn paint(&mut self, ctx: &mut druid::PaintCtx, _: &AppState, _: &druid::Env) {
        const STATE_WIDTH: f64 = 7.0;
        const STATE_SCALE: f64 = 0.5;
        const STATE_COLOR: Color = Color::grey8(200);
        const HOVER_SCALE: f64 = 0.9;
        const HOVER_COLOR: Color = Color::grey8(70);
        const HOVER_RADII: f64 = 0.1;

        let line_style = StrokeStyle {
            line_join: LineJoin::Round,
            line_cap: LineCap::Round,
            dash_pattern: StrokeDash::default(),
            dash_offset: 0.0
        };

        let size = ctx.size().width; // width == height, so whatever

        let hover = *self.hover;
        if hover > 0.0 {

            let hover_len = size * HOVER_SCALE * hover;
            let hover_min = (size - hover_len) / 2.0;
            let hover_max = hover_min + hover_len;
            let shape = RoundedRect::new(hover_min, hover_min, hover_max, hover_max, hover_len * HOVER_RADII);
            ctx.fill(shape, &HOVER_COLOR.with_alpha(hover));
        }
        
        if *self.state.data() {
            let len = size * STATE_SCALE * *self.state;
            if len > 0.0 {
                let off = (size - len) / 2.0;
                let line1 = Line::new((off, off), (off + len, off + len));
                let line2 = Line::new((off + len, off), (off, off + len));
                ctx.stroke_styled(line1, &STATE_COLOR, STATE_WIDTH, &line_style);
                ctx.stroke_styled(line2, &STATE_COLOR, STATE_WIDTH, &line_style);
            }
        } else {
            let off = size / 2.0;
            let rad = STATE_SCALE * off;
            let arc = Arc {
                center: Point::new(off, off),
                radii: druid::Vec2::new(rad, rad),
                start_angle: 0.0,
                sweep_angle: PI * 2.0 * *self.state,
                x_rotation: 0.0,
            };
            ctx.stroke_styled(arc, &STATE_COLOR, STATE_WIDTH, &line_style);
        }
    }
}

pub struct Grid {
    init_anim: Animate<(), EaseInCubic, 1000>,
    done_anim: ReversibleAnimate<u8, EaseInCubic, 500>,
    cells: [WidgetPod<AppState, GridCell>; 9],
    timer: TimerToken
}
impl Default for Grid {
    fn default() -> Self {
        let mut done_anim = ReversibleAnimate::default();
        done_anim.reverse();
        Self {
            init_anim: Default::default(),
            done_anim,
            timer: TimerToken::INVALID,
            cells: [
                WidgetPod::new(GridCell::new(0)),
                WidgetPod::new(GridCell::new(1)),
                WidgetPod::new(GridCell::new(2)),
                WidgetPod::new(GridCell::new(3)),
                WidgetPod::new(GridCell::new(4)),
                WidgetPod::new(GridCell::new(5)),
                WidgetPod::new(GridCell::new(6)),
                WidgetPod::new(GridCell::new(7)),
                WidgetPod::new(GridCell::new(8)),
            ]
        }
    }
}
impl Widget<AppState> for Grid {
    fn event(&mut self, ctx: &mut druid::EventCtx, event: &Event, data: &mut AppState, env: &druid::Env) {
        match event {
            &Event::AnimFrame(t) => {
                ctx.request_paint();
                if !self.init_anim.finished() {
                    self.init_anim.anim_frame(t);
                    ctx.request_anim_frame();
                }
                if !self.done_anim.finished() {
                    self.done_anim.anim_frame(t);
                    ctx.request_anim_frame();
                }
            },
            &Event::Timer(id) if self.timer == id => {
                data.game = TicTacToe::new(State::N);
                ctx.request_update();
            },
            _ => ()
        }

        for e in self.cells.iter_mut() {
            e.event(ctx, event, data, env);
        }
    }

    fn lifecycle(&mut self, ctx: &mut druid::LifeCycleCtx, event: &druid::LifeCycle, data: &AppState, env: &druid::Env) {
        if self.init_anim.starting() {
            ctx.request_anim_frame();
        }

        for e in self.cells.iter_mut() {
            e.lifecycle(ctx, event, data, env);
        }
    }

    fn update(&mut self, ctx: &mut druid::UpdateCtx, old_data: &AppState, data: &AppState, env: &druid::Env) {
        match data.game.done() {
            x @ Some(orien) if x != old_data.game.done() => {
                *self.done_anim.data_mut() = orien;
                self.done_anim.reverse();
                ctx.request_anim_frame();
            },
            x @ None if x != old_data.game.done() => {
                self.done_anim.reverse();
                ctx.request_anim_frame();
            }
            _ => ()
        }

        if data.game.done().is_some() || data.game.draw() {
            self.timer = ctx.request_timer(Duration::from_secs(1));
        }

        for e in self.cells.iter_mut() {
            e.update(ctx, data, env);
        }
    }

    fn layout(&mut self, ctx: &mut druid::LayoutCtx, bc: &druid::BoxConstraints, data: &AppState, env: &druid::Env) -> druid::Size {
        for (i, e) in self.cells.iter_mut().enumerate() {
            let size = e.layout(ctx, &bc.loosen(), data, env);
            e.set_origin(ctx, Point::new((i % 3) as f64 * size.width, (i / 3) as f64 * size.height));
        }
        bc.constrain_aspect_ratio(1.0, 0.0)
    }

    fn paint(&mut self, ctx: &mut druid::PaintCtx, data: &AppState, env: &druid::Env) {
        const GRID_LINE_SCALE: f64 = 0.8;
        const GRID_LINE_COLOR: Color = Color::grey8(175);
        const GRID_LINE_WIDTH: f64 = 5.0;
        const WIN_LINE_SCALE: f64 = 2.0/3.0;
        const WIN_LINE_WIDTH: f64 = 15.0;
        const WIN_LINE_COLOR: Color = Color::grey8(220);

        let line_style = StrokeStyle {
            line_join: LineJoin::Round,
            line_cap: LineCap::Round,
            dash_pattern: StrokeDash::default(),
            dash_offset: 0.0
        };

        let size = ctx.size().width; // width == height, so whatever
        let grid_padd = size / 3.0;
        let grid_line_len = size * GRID_LINE_SCALE * *self.init_anim;
        let grid_line_off = (size - grid_line_len) / 2.0;

        for e in self.cells.iter_mut() {
            e.paint(ctx, data, env);
        }

        // Horizontal Grid Lines
        ctx.stroke_styled(Line {
            p0: Point::new(grid_line_off, grid_padd),
            p1: Point::new(grid_line_off + grid_line_len, grid_padd)
        }, &GRID_LINE_COLOR, GRID_LINE_WIDTH, &line_style);
        ctx.stroke_styled(Line {
            p0: Point::new(grid_line_off, grid_padd * 2.0),
            p1: Point::new(grid_line_off + grid_line_len, grid_padd * 2.0)
        }, &GRID_LINE_COLOR, GRID_LINE_WIDTH, &line_style);

        // Vertical Grid Lines
        ctx.stroke_styled(Line {
            p0: Point::new(grid_padd, grid_line_off),
            p1: Point::new(grid_padd, grid_line_off + grid_line_len)
        }, &GRID_LINE_COLOR, GRID_LINE_WIDTH, &line_style);
        ctx.stroke_styled(Line {
            p0: Point::new(grid_padd * 2.0, grid_line_off),
            p1: Point::new(grid_padd * 2.0, grid_line_off + grid_line_len)
        }, &GRID_LINE_COLOR, GRID_LINE_WIDTH, &line_style);

        // Winning Line
        if *self.done_anim > 0.0 {
            let shape = match *self.done_anim.data() {
                orien @ 0..=2 => {
                    let line_len = size * WIN_LINE_SCALE * *self.done_anim;
                    let line_y = grid_padd * orien as f64 + grid_padd / 2.0;
                    let line_x = (size - line_len) / 2.0;
                    Line::new((line_x, line_y), (line_x + line_len, line_y))
                },
                orien @ 3..=5 => {
                    let line_len = size * WIN_LINE_SCALE * *self.done_anim;
                    let line_x = grid_padd * (orien - 3) as f64 + grid_padd / 2.0;
                    let line_y = (size - line_len) / 2.0;
                    Line::new((line_x, line_y), (line_x, line_y + line_len))
                },
                6 => {
                    let line_len = size * WIN_LINE_SCALE * *self.done_anim;
                    let line_off = (size - line_len) / 2.0;
                    Line::new((line_off, line_off), (line_off + line_len, line_off + line_len))
                },
                7 => {
                    let line_len = size * WIN_LINE_SCALE * *self.done_anim;
                    let line_off = (size - line_len) / 2.0;
                    Line::new((line_off + line_len, line_off), (line_off, line_off + line_len))
                },
                _ => unsafe {unreachable_unchecked()}
            };
            ctx.stroke_styled(shape, &WIN_LINE_COLOR, WIN_LINE_WIDTH, &line_style);
        }
        
    }
}