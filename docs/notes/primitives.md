# primitives in kernelspace
sparkle is (supposed to be!) _capability-based_. what stuff do we provide _from the kernel_?

* a _capability_ is a unforgeable, unique token, representing a relationship (what _methods may be invoked_) to a _kernel object_.
    * capabilities may be copied or moved (_delegated_)
    * given a capability, it is possible to _derive_ a new capability with a subset of the rights of the original capability. this may be used for _partial delegation_.
    * capabilities can be revoked, recursively invalidating all capabilities copied, moved, or derived from the root. (take-grant? anyway, this implies a capability derivation tree.)

## objects
* `IRQ`