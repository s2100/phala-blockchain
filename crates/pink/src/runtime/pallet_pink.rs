pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
    use frame_support::pallet_prelude::*;
    use pallet_contracts::AddressGenerator;
    use sp_core::crypto::UncheckedFrom;
    use sp_runtime::traits::Hash as _;

    type CodeHash<T> = <T as frame_system::Config>::Hash;

    #[pallet::config]
    pub trait Config: frame_system::Config {}

    #[pallet::storage]
    pub(crate) type ClusterId<T: Config> = StorageValue<_, Vec<u8>, ValueQuery>;

    #[pallet::pallet]
    pub struct Pallet<T>(PhantomData<T>);

    impl<T: Config> AddressGenerator<T> for Pallet<T>
    where
        T::AccountId: UncheckedFrom<T::Hash> + AsRef<[u8]>,
    {
        fn generate_address(
            deploying_address: &T::AccountId,
            code_hash: &CodeHash<T>,
            salt: &[u8],
        ) -> T::AccountId {
            let cluster_id = <ClusterId<T>>::get();
            let buf: Vec<_> = deploying_address
                .as_ref()
                .iter()
                .chain(code_hash.as_ref())
                .chain(&cluster_id)
                .chain(salt)
                .cloned()
                .collect();
            UncheckedFrom::unchecked_from(<T as frame_system::Config>::Hashing::hash(
                &buf,
            ))
        }
    }

    impl<T: Config> Pallet<T> {
        pub fn set_cluster_id(cluster_id: &[u8]) {
            <ClusterId<T>>::put(cluster_id.to_vec());
        }
    }
}
