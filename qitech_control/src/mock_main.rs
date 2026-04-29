#[cfg(feature = "mock")]
fn mock_logic(){
    let rt = get_async_runtime();
    let state = Arc::new(SharedAppState::new());
    let _api = rt.spawn(apis::init_api(state.clone()));
    let mut main_state = MainState::new();
    let state_clone = state.clone();
    rt.spawn(start_socketio_queue(state_clone));
    
    let mut starting_dev_addr = 4096;
    let mut meta_subdevices = vec![];

    let mut metas = get_aquapath_meta(starting_dev_addr,0,0);
    let mut idents = get_aquapath_machine_dev_infor(starting_dev_addr);    
    meta_subdevices.extend(metas.subdevices);
    starting_dev_addr += meta_subdevices.len() as u16;
    let end_rx = metas.end_rx;
    let end_tx = metas.end_tx;

    metas = get_extruder_meta(starting_dev_addr,end_tx,end_rx);
    idents.extend(get_extruder_machine_dev_infor(starting_dev_addr));
    meta_subdevices.extend(metas.subdevices);
    starting_dev_addr = starting_dev_addr + meta_subdevices.len() as u16;
    let end_rx = metas.end_rx;
    let end_tx = metas.end_tx;

    metas = get_winder_meta(starting_dev_addr,end_tx,end_rx);
    idents.extend(get_winder_machine_dev_infor(starting_dev_addr));
    meta_subdevices.extend(metas.subdevices);
    starting_dev_addr = starting_dev_addr + meta_subdevices.len() as u16;

    let eth_control = qitech_lib::ethercat_hal::init_ethercat_mock(meta_subdevices,None);
    let mut ecat_handle = eth_control.app_handle;
    let ecat_channel = eth_control.channel;
    let ecat_controller = eth_control.controller;
    
    let mut mapped_ecat_devices = vec![];
    for i in 0..ecat_controller.subdevice_count {
        let meta = ecat_controller.subdevices[i];
        println!("{:?}",meta.get_name());
        let dev = device_from_subdevice_identity_rc(meta).unwrap();
        main_state.subdevices.push(dev.clone());
        mapped_ecat_devices.push((meta, dev));
    }
    main_state.generate_machine_hardware_from_ethercat(idents.clone(), mapped_ecat_devices,ecat_channel.clone());
    let _res = state.fill_ethercat_metadata(ecat_controller.clone(), Some(idents) );

    for key in main_state.hardware.keys() {
        println!("{:?}",key);
        let result = MACHINE_REGISTRY
            .new_machine(key.clone(), main_state.hardware.get(key).unwrap().clone());
        match result {
            Ok(machine) => {
                let _res = state.add_machine_sync(
                    key.clone().into(),
                    None,
                    Some(machine.get_api_sender()),
                );
                main_state.machines.push(machine);
            }
            Err(e) => {
                println!("{:?}", e);
                main_state.machine_errors.insert(*key, e.to_string());
            }
        };
    }

    let state_clone = state.clone();
    rt.spawn(async move {
        let _res = state_clone.send_ethercat_setup_done().await;
        let _res = state_clone.send_machines_event().await;
    });

    loop {
        write_ecat_inputs(
            &mut ecat_handle,
            ecat_controller.clone(),
            main_state.subdevices.clone(),
        );
        run_machines(&mut main_state.machines, &mut main_state.machine_data_reg);
        write_ecat_outputs(
            &mut ecat_handle,
            ecat_controller.clone(),
            main_state.subdevices.clone(),
        );
        std::thread::sleep(Duration::from_micros(10));
    }

}