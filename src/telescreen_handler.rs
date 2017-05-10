use slack::{Event, RtmClient, EventHandler, Message};
use router::{Router,Rule};

pub struct TelescreenHandler {
    router: Router,
}

impl TelescreenHandler {
    pub fn new(router: Router) -> TelescreenHandler {
        TelescreenHandler { router: router }
    }

    pub fn send_message(&self, cli: &RtmClient, unwrapped_channel_name: &str, unwrapped_source_user_name: &str, source_text: &str) {
        let rules: &Vec<Rule> = self.router.rules.as_ref();
        for rule in rules {
            if rule.regex.is_match(unwrapped_channel_name) {
                let dest_channel_id = cli.start_response().channels.as_ref()
                    .and_then(|channels| {
                        channels.iter().find(|chan| match chan.name {
                            None => false,
                            Some(ref name) => name == &(rule.destination),
                        })
                    })
                    .and_then(|chan| chan.id.as_ref());

                let dest_channel_id_unwrap = match dest_channel_id {
                    None => { warn!("No channel: {:?}", dest_channel_id); return },
                    Some(c) => c,
                };

                if unwrapped_channel_name != &(rule.destination) {
                    let message = format!("{:} [ #{} ]: {:}", unwrapped_source_user_name, unwrapped_channel_name, source_text);
                    info!("MESSAGE: {:?}", message);
                    let _ = cli.sender().send_message(&dest_channel_id_unwrap, &message);
                }
            }
        }
    }
}

#[allow(unused_variables)]
impl EventHandler for TelescreenHandler {
    fn on_event(&mut self, cli: &RtmClient, event: Event) {
        debug!("EVENT: {:?}", event);

        match event {
            Event::Message(event) => {
                match *event {
                    Message::Standard(message) => {
                        let source_user_id = match message.user {
                            None => { warn!("No user: {:?}", message.user); return },
                            Some(u) => u,
                        };
                        let source_channel_id = match message.channel {
                            None => { warn!("No channel: {:?}", message.channel); return },
                            Some(c) => c,
                        };
                        let source_text = match message.text {
                            None => { warn!("No text: {:?}", message.text); return },
                            Some(t) => t,
                        };

                        let source_user_name = cli.start_response()
                            .users
                            .as_ref()
                            .and_then(|users| {
                                users.iter().find(|user| match user.id {
                                    None => false,
                                    Some(ref name) => name.to_string() == source_user_id,
                                })
                            }).and_then(|user| user.name.as_ref());

                        let unwrapped_source_user_name = match source_user_name {
                            None => { warn!("No user: {:?}", source_user_name); return },
                            Some(t) => t,
                        };

                        let channel_name = cli.start_response().channels.as_ref()
                            .and_then(|channels| {
                                channels.iter().find(|chan| match chan.id {
                                    None => false,
                                    Some(ref id) => { id == &source_channel_id },
                                })
                            })
                            .and_then(|chan| chan.name.as_ref());

                        let unwrapped_channel_name = match channel_name {
                            None => { warn!("No channel: {:?}", source_channel_id); return },
                            Some(c) => c,
                        };

                        self.send_message(cli, unwrapped_channel_name, unwrapped_source_user_name, &source_text);
                    },
                    _ => { /* noop */ },
                }
            },
            _ => { /* noop */ },
        }
    }

    fn on_close(&mut self, cli: &RtmClient) {
        info!("Disconnected");
    }

    fn on_connect(&mut self, cli: &RtmClient) {
        info!("Connected");
    }
}
