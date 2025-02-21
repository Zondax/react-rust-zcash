package expo.modules.myrustmodule

import expo.modules.kotlin.modules.Module
import expo.modules.kotlin.modules.ModuleDefinition
import android.util.Log

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

    // Native method declaration
    private external fun calculateFee(nTxin: Int, nTxout: Int, nSpend: Int, nSout: Int): Long

    override fun definition() = ModuleDefinition {
        Name("MyRustModule")

        Constants(
            "PI" to Math.PI
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
