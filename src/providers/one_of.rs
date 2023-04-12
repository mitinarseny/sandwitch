macro_rules! provider_one_of {
    ($vis:vis enum $name:ident<project = $proj:ident>($( $t:ident ),+$(,)?)) => {
        #[derive(thiserror::Error, Debug)]
        #[pin_project::pin_project(project = $proj)]
        $vis enum $name<$( $t ),+> {
            $(
                #[error(transparent)]
                $t(#[pin] $t),
            )+
        }

        impl<$( $t ),+> From<$name<$( $t ),+>> for ::ethers::providers::ProviderError
        where
            $($t: Into<::ethers::providers::ProviderError>,)+
        {
            fn from(e: $name<$( $t ),+>) -> Self {
                match e {
                    $(
                        $name::$t(e) => e.into(),
                    )+
                }
            }
        }

        impl<$( $t ),+> ::ethers::providers::RpcError for $name<$( $t ),+>
        where
            $($t: ethers::providers::RpcError,)+
        {
            fn as_error_response(&self) -> Option<&ethers::providers::JsonRpcError> {
                match self {
                    $(
                        Self::$t(e) => e.as_error_response(),
                    )+
                }
            }

            fn as_serde_error(&self) -> Option<&serde_json::Error> {
                match self {
                    $(
                        Self::$t(e) => e.as_serde_error(),
                    )+
                }
            }
        }

        impl<$( $t ),+> ::ethers::providers::JsonRpcClient for $name<$( $t ),+>
        where
            $($t: ::ethers::providers::JsonRpcClient,)+
        {
            type Error = $name<$($t::Error),+>;

            fn request<'life0, 'life1, 'async_trait, T, R>(
                &'life0 self,
                method: &'life1 str,
                params: T,
            ) -> ::core::pin::Pin<Box<dyn ::core::future::Future<Output = Result<R, Self::Error>>
                    + ::core::marker::Send + 'async_trait>>
            where
                T: ::core::fmt::Debug + ::serde::Serialize + Send + Sync,
                R: ::serde::de::DeserializeOwned + ::core::marker::Send,
                T: 'async_trait,
                R: 'async_trait,
                'life0: 'async_trait,
                'life1: 'async_trait,
                Self: 'async_trait,
            {
                match self {
                    $(
                        Self::$t(p) => {
                            ::futures::future::FutureExt::boxed(
                                ::futures::future::TryFutureExt::map_err(
                                    p.request(method, params),
                                    <Self::Error>::$t,
                                )
                            )
                        },
                    )+
                }
            }
        }

        impl<$( $t ),+> futures::stream::Stream for $name<$( $t ),+>
        where
            $($t: ::futures::stream::Stream<Item = Box<::serde_json::value::RawValue>>,)+
        {
            type Item = Box<::serde_json::value::RawValue>;

            fn poll_next(
                self: ::core::pin::Pin<&mut Self>,
                cx: &mut ::core::task::Context<'_>,
            ) -> ::core::task::Poll<Option<Self::Item>> {
                match self.project() {
                    $(
                        $proj::$t(s) => s.poll_next(cx),
                    )+
                }
            }

            #[inline]
            fn size_hint(&self) -> (usize, Option<usize>) {
                match self {
                    $(
                        Self::$t(s) => s.size_hint(),
                    )+
                }
            }
        }

        impl<$( $t ),+> ::ethers::providers::PubsubClient for $name<$( $t ),+>
        where
            $($t: ::ethers::providers::PubsubClient,)+
        {
            type NotificationStream = $name<$($t::NotificationStream),+>;

            fn subscribe<T: Into<::ethers::types::U256>>(&self, id: T) ->
                Result<Self::NotificationStream, Self::Error> {
                match self {
                    $(
                        Self::$t(c) => {
                            c.subscribe(id)
                                .map(<Self::NotificationStream>::$t)
                                .map_err(<Self::Error>::$t)
                        },
                    )+
                }
            }

            fn unsubscribe<T: Into<::ethers::types::U256>>(&self, id: T) -> Result<(), Self::Error> {
                match self {
                    $(
                        Self::$t(c) => c.unsubscribe(id).map_err(<Self::Error>::$t),
                    )+
                }
            }
        }
    };
}

provider_one_of!(pub enum OneOf <project = OneOfProj> (P1, P2));
provider_one_of!(pub enum OneOf3<project = OneOf3Proj>(P1, P2, P3));
provider_one_of!(pub enum OneOf4<project = OneOf4Proj>(P1, P2, P3, P4));
provider_one_of!(pub enum OneOf5<project = OneOf5Proj>(P1, P2, P3, P4, P5));
