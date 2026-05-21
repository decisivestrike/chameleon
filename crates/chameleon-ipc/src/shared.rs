pub static KEYBOARD_LAYOUT: LazyLock<WatchWorker<String>> =
    LazyLock::new(|| match &*COMPOSITOR {
        CompositorVariant::Hyprland(hyprland) => {
            let devices = RUNTIME.block_on(hyprland.devices()).unwrap();
            let active_keymap = devices
                .keyboards
                .into_iter()
                .find(|kb| kb.main)
                .unwrap()
                .active_keymap;

            WatchWorker::new(active_keymap, {
                let mut receiver = hyprland.subscribe();
                async move |sender| {
                    loop {
                        if let Ok(event) = receiver.recv().await
                            && let HyprEvent::ActiveLayout {
                                layout_name, ..
                            } = event
                        {
                            if let Err(e) = sender.send(layout_name) {
                                error!("{}", e);
                            }
                        }
                    }
                }
            })
        }
        CompositorVariant::Niri(niri) => unreachable!(),
        CompositorVariant::Unknown => panic!("U should use Hyprland or Niri"),
    });
