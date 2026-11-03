//! Ely.by credentials are entered only in the system browser.
use super::*;

#[derive(Default)]
pub(super) struct ElyUi {
    pub active: bool,
}

impl Launcher {
    pub(super) fn submit_ely(&mut self) {
        if matches!(
            self.sign_in_state,
            Some(
                SignInState::Starting
                    | SignInState::Browser(_)
                    | SignInState::Working(_)
                    | SignInState::Code(_)
            )
        ) {
            return;
        }
        let expected = self.sign_in_target.clone().filter(account::Account::is_ely);
        self.cancel_sign_in();
        self.ely.active = true;
        self.sign_in_target = expected.clone();
        self.sign_in = Some(SignIn::start_ely(expected));
        self.sign_in_state = Some(SignInState::Starting);
    }

    pub(super) fn ely_form(&self, cx: &mut Context<Self>) -> Div {
        let t = self.theme;
        let mut body = div().flex().flex_col().gap(px(8.));
        if let Some(target) = self.sign_in_target.as_ref().filter(|a| a.is_ely()) {
            body =
                body.child(div().child(trf("Sign in again as {name}", &[("name", &target.name)])));
        }
        let busy = self.ely.active
            && matches!(
                self.sign_in_state,
                Some(SignInState::Starting | SignInState::Browser(_) | SignInState::Working(_))
            );
        if self.ely.active {
            body = match &self.sign_in_state {
                Some(SignInState::Done(Err(e))) => {
                    body.child(div().text_color(t.danger).child(e.clone()))
                }
                Some(SignInState::Done(Ok(a))) => {
                    body.child(div().child(trf("Signed in as {name}.", &[("name", &a.name)])))
                }
                Some(SignInState::Starting | SignInState::Browser(_) | SignInState::Working(_)) => {
                    body.child(div().child(tr("Waiting for browser sign-in…")))
                }
                _ => body,
            };
        }
        if busy {
            if let Some(SignInState::Browser(url)) = &self.sign_in_state {
                let url = url.clone();
                body = body.child(
                    button("ely-browser", "Open link", Kind::Secondary, &t)
                        .on_click(move |_, _, cx| cx.open_url(&url)),
                );
            }
            body = body.child(button("ely-cancel", "Cancel", Kind::Ghost, &t).on_click(
                cx.listener(|this, _: &ClickEvent, _, cx| {
                    this.cancel_sign_in();
                    cx.notify();
                }),
            ));
        } else {
            body = body.child(
                button("ely-sign-in", "Sign in with Ely.by", Kind::Secondary, &t).on_click(
                    cx.listener(|this, _: &ClickEvent, _, cx| {
                        this.submit_ely();
                        cx.notify();
                    }),
                ),
            );
        }
        body.child(button("ely-manage", "Manage Ely.by skin and account", Kind::Ghost, &t)
            .on_click(|_, _, cx| cx.open_url("https://account.ely.by/")))
            .child(text(theme::CAPTION).text_color(t.muted).child(tr("Sign in on the Ely.by website. The launcher never receives your password or two-factor code.")))
    }
}
