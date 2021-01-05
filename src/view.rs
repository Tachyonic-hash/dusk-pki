// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use crate::keys::spend::stealth;

use crate::Error;
use crate::{
    permutation, JubJubAffine, JubJubExtended, JubJubScalar, PublicSpendKey, SecretSpendKey,
};

use dusk_jubjub::GENERATOR_EXTENDED;
use subtle::{Choice, ConstantTimeEq};

use core::convert::TryFrom;
use core::fmt;

/// Pair of a secret `a` and public `b·G`
///
/// The notes are encrypted against secret a, so this is used to decrypt the
/// blinding factor and value
#[derive(Debug, Clone, Copy)]
pub struct ViewKey {
    a: JubJubScalar,
    B: JubJubExtended,
}

impl ConstantTimeEq for ViewKey {
    fn ct_eq(&self, other: &Self) -> Choice {
        // TODO - Why self.a is not checked?
        self.B.ct_eq(&other.B)
    }
}

impl PartialEq for ViewKey {
    fn eq(&self, other: &Self) -> bool {
        self.a == other.a && self.ct_eq(&other).into()
    }
}

impl Eq for ViewKey {}

impl Default for ViewKey {
    fn default() -> Self {
        SecretSpendKey::default().view_key()
    }
}

impl ViewKey {
    /// This method is used to construct a new `ViewKey` from the given
    /// pair of secret `a` and public `b·G`.
    pub fn new(a: JubJubScalar, B: JubJubExtended) -> Self {
        Self { a, B }
    }

    /// Derive the secret to deterministically construct a [`PublicSpendKey`]
    pub fn public_spend_key(&self) -> PublicSpendKey {
        let A = GENERATOR_EXTENDED * self.a;

        PublicSpendKey::new(A, self.B)
    }

    /// Gets `a`
    pub fn a(&self) -> &JubJubScalar {
        &self.a
    }

    /// Gets `B` (`b·G`)
    pub fn B(&self) -> &JubJubExtended {
        &self.B
    }

    /// Checks `PKr = H(R · a) · G + B`
    pub fn owns(&self, owner: &impl stealth::Ownable) -> bool {
        let sa = owner.stealth_address();

        let aR = sa.R() * self.a();
        let aR = permutation::hash(&aR);
        let aR = GENERATOR_EXTENDED * aR;
        let pk_r = aR + self.B();

        sa.pk_r() == &pk_r
    }
}

impl From<SecretSpendKey> for ViewKey {
    fn from(secret: SecretSpendKey) -> Self {
        secret.view_key()
    }
}

impl From<&SecretSpendKey> for ViewKey {
    fn from(secret: &SecretSpendKey) -> Self {
        secret.view_key()
    }
}

impl From<&ViewKey> for [u8; 64] {
    fn from(vk: &ViewKey) -> Self {
        let mut bytes = [0u8; 64];
        bytes[..32].copy_from_slice(&vk.a.to_bytes()[..]);
        bytes[32..].copy_from_slice(&JubJubAffine::from(&vk.B).to_bytes()[..]);
        bytes
    }
}

impl TryFrom<&str> for ViewKey {
    type Error = Error;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        use crate::decode::decode;

        if s.len() != 128 {
            return Err(Error::BadLength {
                found: s.len(),
                expected: 128,
            });
        }

        let a = hex::decode(&s[..64]).map_err(|_| Error::InvalidPoint)?;
        let a = decode::<JubJubScalar>(&a[..])?;

        let B = hex::decode(&s[64..]).map_err(|_| Error::InvalidPoint)?;
        let B = JubJubExtended::from(decode::<JubJubAffine>(&B[..])?);

        Ok(ViewKey::new(a, B))
    }
}

impl fmt::LowerHex for ViewKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let bytes: [u8; 64] = self.into();

        if f.alternate() {
            write!(f, "0x")?
        }

        for byte in &bytes[..] {
            write!(f, "{:02X}", &byte)?
        }

        Ok(())
    }
}

impl fmt::UpperHex for ViewKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let bytes: [u8; 64] = self.into();

        if f.alternate() {
            write!(f, "0x")?
        }

        for byte in &bytes[..] {
            write!(f, "{:02X}", &byte)?
        }

        Ok(())
    }
}

impl fmt::Display for ViewKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:x}", self)
    }
}
