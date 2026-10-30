package n.noah.mira

import org.junit.Assert.assertArrayEquals
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNotEquals
import org.junit.Assert.assertNull
import org.junit.Assert.assertTrue
import org.junit.Assert.fail
import org.junit.Test

private class MemoryStorage(var sealed: ByteArray? = null) : WrappedKeyStorage {
    override fun read(): ByteArray? = sealed
    override fun write(sealed: ByteArray) { this.sealed = sealed }
    override fun clear() { sealed = null }
}

/** Faux Keystore : une clé d'enveloppe numérotée ; la jeter en fait naître une autre, qui ne rouvre pas l'ancien contenu. */
private class FakeWrapper(var wrapFails: Boolean = false, var shortKey: Boolean = false) : KeyWrapper {
    var generation = 1
    var discards = 0

    override fun wrap(plain: ByteArray): ByteArray {
        if (wrapFails) throw IllegalStateException("Keystore inutilisable")
        return byteArrayOf(generation.toByte()) + plain
    }

    override fun unwrap(sealed: ByteArray): ByteArray {
        if (sealed[0].toInt() != generation) throw IllegalStateException("VERIFICATION_FAILED")
        val plain = sealed.copyOfRange(1, sealed.size)
        return if (shortKey) plain.copyOf(5) else plain
    }

    override fun discard() { discards++; generation++ }
}

private var counter = 0
private fun nextKey() = ByteArray(SecretsKey.KEY_SIZE) { (++counter).toByte() }

class SecretsKeyTest {
    @Test fun firstUseCreatesAndStoresTheKey() {
        val storage = MemoryStorage()
        val key = SecretsKey(storage, FakeWrapper(), ::nextKey).get()
        assertEquals(SecretsKey.KEY_SIZE, key.size)
        assertTrue(storage.sealed != null)
    }

    @Test fun laterUsesReturnTheSameKey() {
        val storage = MemoryStorage()
        val wrapper = FakeWrapper()
        val first = SecretsKey(storage, wrapper, ::nextKey).get()
        val second = SecretsKey(storage, wrapper, ::nextKey).get()
        assertArrayEquals(first, second)
        assertEquals(0, wrapper.discards)
    }

    @Test fun anUnreadableKeyIsReplacedInsteadOfBlockingTheEngine() {
        val storage = MemoryStorage()
        val wrapper = FakeWrapper()
        val old = SecretsKey(storage, wrapper, ::nextKey).get()
        wrapper.generation++
        var reported: Exception? = null
        val fresh = SecretsKey(storage, wrapper, ::nextKey) { reported = it }.get()
        assertEquals(SecretsKey.KEY_SIZE, fresh.size)
        assertTrue(!old.contentEquals(fresh))
        assertEquals(1, wrapper.discards)
        assertEquals("VERIFICATION_FAILED", reported?.message)
    }

    @Test fun theReplacementSurvivesTheNextLaunch() {
        val storage = MemoryStorage()
        val wrapper = FakeWrapper()
        SecretsKey(storage, wrapper, ::nextKey).get()
        wrapper.generation++
        val fresh = SecretsKey(storage, wrapper, ::nextKey).get()
        val again = SecretsKey(storage, wrapper, ::nextKey).get()
        assertArrayEquals(fresh, again)
        assertEquals(1, wrapper.discards)
    }

    @Test fun aKeyOfTheWrongSizeCountsAsUnreadable() {
        val storage = MemoryStorage()
        val wrapper = FakeWrapper()
        SecretsKey(storage, wrapper, ::nextKey).get()
        wrapper.shortKey = true
        val fresh = SecretsKey(storage, wrapper, ::nextKey).get()
        assertEquals(SecretsKey.KEY_SIZE, fresh.size)
        assertEquals(1, wrapper.discards)
    }

    @Test fun aKeystoreThatCannotWrapFailsWithoutStoringAnything() {
        val storage = MemoryStorage()
        try {
            SecretsKey(storage, FakeWrapper(wrapFails = true), ::nextKey).get()
            fail("l'échec du Keystore doit remonter")
        } catch (e: IllegalStateException) {
            assertEquals("Keystore inutilisable", e.message)
        }
        assertNull(storage.sealed)
    }
}
