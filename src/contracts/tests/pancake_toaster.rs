use contracts::pancake_toaster;

#[test]
fn pancake() {
    pancake_toaster::pancake_toaster::PancakeToaster::deploy(client, ());
}
