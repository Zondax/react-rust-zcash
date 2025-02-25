package expo.modules.myrustmodule

import expo.modules.kotlin.modules.Module
import expo.modules.kotlin.modules.ModuleDefinition
import android.util.Log

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
    private external fun getErrorDescription(errorCode: Int): String
    
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
        AsyncFunction("buildTransaction") { builderId: Double, spendPath: String, outputPath: String, txVersion: Int ->
            try {
                Log.d("MyRustModule", "Building transaction with builder $builderId")
                
                val txBytes = buildTransaction(builderId.toLong(), spendPath, outputPath, txVersion)
                val hexString = txBytes.joinToString("") { "%02x".format(it) }
                
                Log.d("MyRustModule", "Transaction built successfully, size: ${txBytes.size} bytes")
                hexString
            } catch (e: Exception) {
                Log.e("MyRustModule", "Error building transaction", e)
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
