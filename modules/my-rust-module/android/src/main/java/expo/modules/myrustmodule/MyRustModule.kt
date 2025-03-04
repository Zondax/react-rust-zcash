package expo.modules.myrustmodule

import expo.modules.kotlin.modules.Module
import expo.modules.kotlin.modules.ModuleDefinition
import android.util.Log
import java.io.File
import java.io.IOException

// JNI helper classes
class TransparentInput(
    val outp: String,
    val pk: String,
    val address: String,
    val value: Long
)

class TransparentOutput(
    val address: String,
    val value: Long
)

data class TinData(
    val path: LongArray,        // Array of 5 u32 values stored as Long in Kotlin
    val address: String,        // Hex-encoded string representing a Script
    val value: Long             // u64 value representing an Amount
) {
    // Override equals and hashCode since LongArray doesn't have built-in equality
    override fun equals(other: Any?): Boolean {
        if (this === other) return true
        if (javaClass != other?.javaClass) return false

        other as TinData

        if (!path.contentEquals(other.path)) return false
        if (address != other.address) return false
        if (value != other.value) return false

        return true
    }

    override fun hashCode(): Int {
        var result = path.contentHashCode()
        result = 31 * result + address.hashCode()
        result = 31 * result + value.hashCode()
        return result
    }
}

data class ToutData(
    val address: String,        // Hex-encoded string representing a Script
    val value: Long             // u64 value representing an Amount
)

data class SaplingInData(
    val path: Long,              // Single u32 value
    val address: String,        // Hex-encoded string representing a PaymentAddress
    val value: Long             // u64 value representing an Amount
)

data class SaplingOutData(
    val address: String,        // Hex-encoded string representing a PaymentAddress
    val value: Long,            // u64 value representing an Amount
    val memoType: Byte,         // Single byte value
    val hasOvk: Boolean,        // Boolean indicating if ovk is present
    val ovk: ByteArray? = null  // Optional byte array (32 bytes) representing an OutgoingViewingKey
)

data class InitData(
    val tIn: List<TinData>,
    val tOut: List<ToutData>,
    val sSpend: List<SaplingInData>,
    val sOutput: List<SaplingOutData>
)

data class Signatures(
    var transparentSigs: List<String> = listOf(),
    var saplingSigs: List<String> = listOf()
)

// Convert a hex string to a ByteArray
fun hexToByteArray(hex: String): ByteArray {
    val hexString = if (hex.startsWith("0x")) hex.substring(2) else hex
    val len = hexString.length
    val data = ByteArray(len / 2)
    
    for (i in 0 until len step 2) {
        data[i / 2] = ((Character.digit(hexString[i], 16) shl 4) + 
                        Character.digit(hexString[i + 1], 16)).toByte()
    }
    
    return data
}


class MyRustModule : Module() {
    companion object {
        init {
            try {
                System.loadLibrary("native_rust_lib")
                Log.d("MyRustModule", "Native library loaded successfully")
            } catch (e: UnsatisfiedLinkError) {
                Log.e("MyRustModule", "Failed to load native library", e)
            }
        }
    }
    
    // Native method declarations
    private external fun calculateFee(nTxin: Int, nTxout: Int, nSpend: Int, nSout: Int): Long
    private external fun createBuilder(fee: Long, networkType: Int): Long
    private external fun destroyBuilder(builderId: Long): Int
    private external fun addTransparentInput(builderId: Long, input: TransparentInput): Int
    private external fun addTransparentOutput(builderId: Long, output: TransparentOutput): Int
    private external fun buildTransaction(builderId: Long, spendPath: String, outputPath: String, txVersion: Int): ByteArray
    private external fun finalizeTransaction(builderId: Long): ByteArray
    private external fun getErrorDescription(errorCode: Int): String
    private external fun getInitTxData(initData: InitData): ByteArray
    private external fun addSignatures(builderId: Long, signatures: Signatures): Int
    
    override fun definition() = ModuleDefinition {
        Name("MyRustModule")
        
        Constants(
            "NETWORK_MAINNET" to 0,
            "NETWORK_TESTNET" to 1
        )
        
        // Expose the calculate fee function to JavaScript
        AsyncFunction("calculateFee") { nTxin: Int, nTxout: Int, nSpend: Int, nSout: Int ->
            try {
                Log.d("MyRustModule", "Calling native calculateFee: $nTxin, $nTxout, $nSpend, $nSout")
                val result = calculateFee(nTxin, nTxout, nSpend, nSout)
                Log.d("MyRustModule", "Native calculateFee result: $result")
                result.toDouble() // Convert to Double for JavaScript
            } catch (e: Exception) {
                Log.e("MyRustModule", "Error in calculateFee", e)
                throw e
            }
        }
        
        // Create transaction builder
        AsyncFunction("createBuilder") { fee: Double, networkType: Int ->
            try {
                Log.d("MyRustModule", "Creating builder with fee: $fee, networkType: $networkType")
                val result = createBuilder(fee.toLong(), networkType)
                Log.d("MyRustModule", "Builder created with ID: $result")
                result.toDouble() // Convert to Double for JavaScript
            } catch (e: Exception) {
                Log.e("MyRustModule", "Error creating builder", e)
                throw e
            }
        }
        
        // Destroy transaction builder
        AsyncFunction("destroyBuilder") { builderId: Double ->
            try {
                Log.d("MyRustModule", "Destroying builder: $builderId")
                val result = destroyBuilder(builderId.toLong())
                Log.d("MyRustModule", "Builder destroy result: $result")
                result
            } catch (e: Exception) {
                Log.e("MyRustModule", "Error destroying builder", e)
                throw e
            }
        }
        
        // Add transparent input
        AsyncFunction("addTransparentInput") { builderId: Double, input: Map<String, Any> ->
            try {
                val outpoint = input["outp"] as? String ?: ""
                val publicKey = input["pk"] as? String ?: ""
                val address = input["address"] as? String ?: ""
                val value = (input["value"] as? Double ?: 0.0).toLong()
                
                Log.d("MyRustModule", "Adding transparent input to builder $builderId: $address, value: $value")
                
                // Create TransparentInput object for JNI
                val inputObj = TransparentInput(outpoint, publicKey, address, value)
                
                val result = addTransparentInput(builderId.toLong(), inputObj)
                Log.d("MyRustModule", "Add transparent input result: $result")
                result
            } catch (e: Exception) {
                Log.e("MyRustModule", "Error adding transparent input", e)
                throw e
            }
        }
        
        // Add transparent output
        AsyncFunction("addTransparentOutput") { builderId: Double, output: Map<String, Any> ->
            try {
                val address = output["address"] as? String ?: ""
                val value = (output["value"] as? Double ?: 0.0).toLong()
                
                Log.d("MyRustModule", "Adding transparent output to builder $builderId: $address, value: $value")
                
                // Create TransparentOutput object for JNI
                val outputObj = TransparentOutput(address, value)
                
                val result = addTransparentOutput(builderId.toLong(), outputObj)
                Log.d("MyRustModule", "Add transparent output result: $result")
                result
            } catch (e: Exception) {
                Log.e("MyRustModule", "Error adding transparent output", e)
                throw e
            }
        }
        
        // Build transaction
        // Build transaction
        AsyncFunction("buildTransaction") { builderId: Double, _: String, _: String, txVersion: Int ->
            try {
                Log.d("MyRustModule", "Building transaction with builder $builderId")
                
                // Get application context
                val context = appContext.reactContext ?: throw Exception("No React context available")
                // Get application context
                
                // Define paths to extracted files in the app's internal storage
                val filesDir = context.filesDir
                val spendPath = File(filesDir, "sapling-spend.params").absolutePath
                val outputPath = File(filesDir, "sapling-output.params").absolutePath
                
                // Check if files exist in internal storage
                val spendFile = File(spendPath)
                val outputFile = File(outputPath)
                
                // If files don't exist, extract them from assets
                if (!spendFile.exists()) {
                    context.assets.open("zcash-params/sapling-spend.params").use { input ->
                        File(spendPath).outputStream().use { output ->
                            input.copyTo(output)
                        }
                    }
                    Log.d("MyRustModule", "Extracted sapling-spend.params to $spendPath")
                }
                
                if (!outputFile.exists()) {
                    context.assets.open("zcash-params/sapling-output.params").use { input ->
                        File(outputPath).outputStream().use { output ->
                            input.copyTo(output)
                        }
                    }
                    Log.d("MyRustModule", "Extracted sapling-output.params to $outputPath")
                }
                
                // Now call the native method with the extracted file paths
                Log.d("MyRustModule", "Calling native buildTransaction with paths: $spendPath, $outputPath")
                val txBytes = buildTransaction(builderId.toLong(), spendPath, outputPath, txVersion)
                val hexString = txBytes.joinToString("") { "%02x".format(it) }
                
                Log.d("MyRustModule", "Transaction built successfully, size: ${hexString.length / 2} bytes")
                hexString
            } catch (e: Exception) {
                Log.e("MyRustModule", "Error building transaction", e)
                throw e
            }
        }
        // Initialize transaction data
        AsyncFunction("getInitTxData") { initData: Map<String, Any> ->
            try {
                Log.d("MyRustModule", "Calling getInitTxData with init data")
                
                // Parse transparent inputs
                val tInList = (initData["tIn"] as? List<*>)?.mapNotNull { input ->
                    (input as? Map<*, *>)?.let { inputMap ->
                        // Parse path array (convert to LongArray)
                        val pathList = (inputMap["path"] as? List<*>)?.mapNotNull { 
                            (it as? Number)?.toLong() 
                        } ?: listOf<Long>()
                        
                        // Create a fixed-size LongArray of 5 elements
                        val pathArray = LongArray(5)
                        for (i in 0 until minOf(pathList.size, 5)) {
                            pathArray[i] = pathList[i]
                        }
                        
                        TinData(
                            path = pathArray,
                            address = inputMap["address"] as? String ?: "",
                            value = ((inputMap["value"] as? Number) ?: 0).toLong()
                        )
                    }
                } ?: listOf()
                
                // Parse transparent outputs - unchanged
                val tOutList = (initData["tOut"] as? List<*>)?.mapNotNull { output ->
                    (output as? Map<*, *>)?.let { outputMap ->
                        ToutData(
                            address = outputMap["address"] as? String ?: "",
                            value = ((outputMap["value"] as? Number) ?: 0).toLong()
                        )
                    }
                } ?: listOf()
                
                // Parse Sapling inputs - unchanged
               val sSpendList = (initData["sSpend"] as? List<*>)?.mapNotNull { spend ->
                    (spend as? Map<*, *>)?.let { spendMap ->
                        SaplingInData(
                            path = (spendMap["path"] as? Number)?.toLong() ?: 0L,  // Convert to Long instead of Int
                            address = spendMap["address"] as? String ?: "",
                            value = ((spendMap["value"] as? Number) ?: 0).toLong()
                        )
                    }
                } ?: listOf() 

                // Parse Sapling outputs - unchanged
                val sOutputList = (initData["sOutput"] as? List<*>)?.mapNotNull { output ->
                    (output as? Map<*, *>)?.let { outputMap ->
                        val hasOvk = outputMap["hasOvk"] as? Boolean ?: false
                        var ovk: ByteArray? = null
                        
                        if (hasOvk) {
                            // Parse OutgoingViewingKey if present
                            (outputMap["ovk"] as? String)?.let { ovkString ->
                                // Convert hex string to ByteArray if needed
                                ovk = hexToByteArray(ovkString)
                            }
                        }
                        
                        SaplingOutData(
                            address = outputMap["address"] as? String ?: "",
                            value = ((outputMap["value"] as? Number) ?: 0).toLong(),
                            memoType = ((outputMap["memoType"] as? Number) ?: 0).toByte(),
                            hasOvk = hasOvk,
                            ovk = ovk
                        )
                    }
                } ?: listOf()
                
                // Create the InitData object
                val initDataObj = InitData(
                    tIn = tInList,
                    tOut = tOutList,
                    sSpend = sSpendList,
                    sOutput = sOutputList
                )
                
                // Call the native method
                val dataBytes = getInitTxData(initDataObj)
                val hexString = dataBytes.joinToString("") { "%02x".format(it) }
                
                Log.d("MyRustModule", "getInitTxData completed successfully, size: ${hexString.length / 2} bytes")
                hexString
            } catch (e: Exception) {
                Log.e("MyRustModule", "Error in getInitTxData", e)
                throw e
            }
        }

        AsyncFunction("finalizeTransaction") { builderId: Double ->
            try {
                Log.d("MyRustModule", "Finalizing transaction with builder $builderId")
                
                // Call the native method
                val txBytes = finalizeTransaction(builderId.toLong())
                val hexString = txBytes.joinToString("") { "%02x".format(it) }
                
                Log.d("MyRustModule", "Transaction finalized successfully, size: ${hexString.length / 2} bytes")
                hexString
            } catch (e: Exception) {
                Log.e("MyRustModule", "Error finalizing transaction", e)
                throw e
            }
        }

        // Get error description
        AsyncFunction("getErrorDescription") { errorCode: Int ->
            try {
                val description = getErrorDescription(errorCode)
                Log.d("MyRustModule", "Error description for code $errorCode: $description")
                description
            } catch (e: Exception) {
                Log.e("MyRustModule", "Error getting error description", e)
                throw e
            }
        }
        
        Events("onChange")
        
        AsyncFunction("setValueAsync") { value: String ->
            sendEvent("onChange", mapOf(
                "value" to value
            ))
        }
        
        View(MyRustModuleView::class) {
            Prop("name") { view: MyRustModuleView, prop: String ->
                println(prop)
            }
        }
    }
}
