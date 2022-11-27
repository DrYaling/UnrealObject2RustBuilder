// Copyright Epic Games, Inc. All Rights Reserved.

#pragma once

#include "RenderGraphParameter.h"
#include "RenderGraphTextureSubresource.h"
#include "RendererInterface.h"
#include "RHITransientResourceAllocator.h"

struct FPooledRenderTarget;
class FRenderTargetPool;

/** Used for tracking pass producer / consumer edges in the graph for culling and pipe fencing. */
struct FRDGProducerState
{
	/** Returns whether the next state is dependent on the last producer in the producer graph. */
	static bool IsDependencyRequired(FRDGProducerState LastProducer, ERHIPipeline LastPipeline, FRDGProducerState NextState, ERHIPipeline NextPipeline);

	FRDGProducerState() = default;

	ERHIAccess Access = ERHIAccess::Unknown;
	FRDGPassHandle PassHandle;
	FRDGViewHandle NoUAVBarrierHandle;
};

using FRDGProducerStatesByPipeline = TRHIPipelineArray<FRDGProducerState>;

/** Used for tracking the state of an individual subresource during execution. */
struct FRDGSubresourceState
{
	/** Given a before and after state, returns whether a resource barrier is required. */
	static bool IsTransitionRequired(const FRDGSubresourceState& Previous, const FRDGSubresourceState& Next);

	/** Given a before and after state, returns whether they can be merged into a single state. */
	static bool IsMergeAllowed(ERDGParentResourceType ResourceType, const FRDGSubresourceState& Previous, const FRDGSubresourceState& Next);

	FRDGSubresourceState() = default;

	/** Initializes the first and last pass and the pipeline. Clears any other pass state. */
	void SetPass(ERHIPipeline Pipeline, FRDGPassHandle PassHandle);

	/** Finalizes the state at the end of the transition chain; keeps access intact. */
	void Finalize();

	/** Validates that the state is in a correct configuration for use. */
	void Validate();

	/** Returns whether the state is used by the pipeline. */
	bool IsUsedBy(ERHIPipeline Pipeline) const;

	/** Returns the last pass across either pipe. */
	FRDGPassHandle GetLastPass() const;

	/** Returns the first pass across either pipe. */
	FRDGPassHandle GetFirstPass() const;

	/** Returns the pipeline mask this state is used on. */
	ERHIPipeline GetPipelines() const;

	/** The last used access on the pass. */
	ERHIAccess Access = ERHIAccess::Unknown;

	/** The last used transition flags on the pass. */
	EResourceTransitionFlags Flags = EResourceTransitionFlags::None;

	/** The first pass in this state. */
	FRDGPassHandlesByPipeline FirstPass;

	/** The last pass in this state. */
	FRDGPassHandlesByPipeline LastPass;

	/** The last no-UAV barrier to be used by this subresource. */
	FRDGViewUniqueFilter NoUAVBarrierFilter;
};

using FRDGTextureSubresourceState = TRDGTextureSubresourceArray<FRDGSubresourceState, FDefaultAllocator>;
using FRDGTextureTransientSubresourceState = TRDGTextureSubresourceArray<FRDGSubresourceState, FRDGArrayAllocator>;
using FRDGTextureTransientSubresourceStateIndirect = TRDGTextureSubresourceArray<FRDGSubresourceState*, FRDGArrayAllocator>;

/** Generic graph resource. */
class RENDERCORE_API FRDGResource
{
public:
	FRDGResource(const FRDGResource&) = delete;
	virtual ~FRDGResource() = default;

	// Name of the resource for debugging purpose.
	const TCHAR* const Name = nullptr;

	//////////////////////////////////////////////////////////////////////////
	//! The following methods may only be called during pass execution.

	/** Marks this resource as actually used by a resource. This is to track what dependencies on pass was actually unnecessary. */
#if RDG_ENABLE_DEBUG
	virtual void MarkResourceAsUsed();
#else
	inline  void MarkResourceAsUsed() {}
#endif

	FRHIResource* GetRHI() const
	{
		IF_RDG_ENABLE_DEBUG(ValidateRHIAccess());
		return ResourceRHI;
	}

	//////////////////////////////////////////////////////////////////////////

protected:
	FRDGResource(const TCHAR* InName)
		: Name(InName)
	{}

	FRHIResource* GetRHIUnchecked() const
	{
		return ResourceRHI;
	}

	FRHIResource* ResourceRHI = nullptr;

#if RDG_ENABLE_DEBUG
	void ValidateRHIAccess() const;
#endif

private:
#if RDG_ENABLE_DEBUG
	struct FRDGResourceDebugData* DebugData = nullptr;
	FRDGResourceDebugData& GetDebugData() const;
	bool IsPassthrough() const;
#endif

	friend FRDGBuilder;
	friend FRDGUserValidation;
	friend FRDGBarrierValidation;
};

class FRDGUniformBuffer
	: public FRDGResource
{
public:
	FORCEINLINE const FRDGParameterStruct& GetParameters() const
	{
		return ParameterStruct;
	}

#if RDG_ENABLE_DEBUG
	RENDERCORE_API void MarkResourceAsUsed() override;
#else
	inline         void MarkResourceAsUsed() {}
#endif

	//////////////////////////////////////////////////////////////////////////
	//! The following methods may only be called during pass execution.

	FRHIUniformBuffer* GetRHI() const
	{
		return static_cast<FRHIUniformBuffer*>(FRDGResource::GetRHI());
	}

	//////////////////////////////////////////////////////////////////////////

protected:
	template <typename TParameterStruct>
	explicit FRDGUniformBuffer(const TParameterStruct* InParameters, const TCHAR* InName)
		: FRDGResource(InName)
		, ParameterStruct(InParameters)
	{}

private:

	const FRDGParameterStruct ParameterStruct;
	TRefCountPtr<FRHIUniformBuffer> UniformBufferRHI;
	FRDGUniformBufferHandle Handle;

	friend FRDGBuilder;
	friend FRDGUniformBufferRegistry;
	friend FRDGAllocator;
};

template <typename ParameterStructType>
class TRDGUniformBuffer : public FRDGUniformBuffer
{
public:
	FORCEINLINE const TRDGParameterStruct<ParameterStructType>& GetParameters() const
	{
		return static_cast<const TRDGParameterStruct<ParameterStructType>&>(FRDGUniformBuffer::GetParameters());
	}

	FORCEINLINE TUniformBufferRef<ParameterStructType> GetRHIRef() const
	{
		return TUniformBufferRef<ParameterStructType>(GetRHI());
	}

	FORCEINLINE const ParameterStructType* operator->() const
	{
		return Parameters;
	}

private:
	explicit TRDGUniformBuffer(const ParameterStructType* InParameters, const TCHAR* InName)
		: FRDGUniformBuffer(InParameters, InName)
		, Parameters(InParameters)
	{}

	const ParameterStructType* Parameters;

	friend FRDGBuilder;
	friend FRDGUniformBufferRegistry;
	friend FRDGAllocator;
};

/** A render graph resource with an allocation lifetime tracked by the graph. May have child resources which reference it (e.g. views). */
class RENDERCORE_API FRDGParentResource
	: public FRDGResource
{
public:
	/** The type of this resource; useful for casting between types. */
	const ERDGParentResourceType Type;

	/** Whether this resource is externally registered with the graph (i.e. the user holds a reference to the underlying resource outside the graph). */
	bool IsExternal() const
	{
		return bExternal;
	}

	/** Whether this resource is has been queued for extraction at the end of graph execution. */
	bool IsExtracted() const
	{
		return bExtracted;
	}

	/** Whether a prior pass added to the graph produced contents for this resource. External resources are not considered produced
	 *  until used for a write operation. This is a union of all subresources, so any subresource write will set this to true.
	 */
	bool HasBeenProduced() const
	{
		return bProduced;
	}

	/** Marks a resource as excluded from the transient allocator, if it would otherwise be included. */
	void SetNonTransient()
	{
		check(!bTransient);
		bUserSetNonTransient = 1;
	}

protected:
	FRDGParentResource(const TCHAR* InName, ERDGParentResourceType InType);
	~FRDGParentResource();

	/** Whether this is an externally registered resource. */
	uint8 bExternal : 1;

	/** Whether this is an extracted resource. */
	uint8 bExtracted : 1;

	/** Whether any sub-resource has been used for write by a pass. */
	uint8 bProduced : 1;

	/** Whether this resource is allocated through the transient resource allocator. */
	uint8 bTransient : 1;

	/** Whether this resource is set to be non-transient by the user. */
	uint8 bUserSetNonTransient : 1;

	/** Whether this resource is the last owner of its allocation (i.e. nothing aliases the allocation later in the execution timeline). */
	uint8 bLastOwner : 1;

	/** If true, the resource was not used by any pass not culled by the graph. */
	uint8 bCulled : 1;

	/** If true, the resource has been used on an async compute pass and may have async compute states. */
	uint8 bUsedByAsyncComputePass : 1;

	/** Assigns this resource as a simple passthrough container for an RHI resource. */
	void SetPassthroughRHI(FRHIResource* InResourceRHI);

private:
	/** Number of references in passes and deferred queries. */
	uint16 ReferenceCount = 0;

	/** The initial and final states of the resource assigned by the user, if known. */
	ERHIAccess AccessInitial = ERHIAccess::Unknown;
	ERHIAccess AccessFinal = ERHIAccess::Unknown;

	FRDGPassHandle FirstPass;
	FRDGPassHandle LastPass;

#if RDG_ENABLE_TRACE
	uint16 TraceOrder = 0;
	TArray<FRDGPassHandle, FRDGArrayAllocator> TracePasses;
#endif

#if RDG_ENABLE_DEBUG
	struct FRDGParentResourceDebugData* ParentDebugData = nullptr;
	FRDGParentResourceDebugData& GetParentDebugData() const;
#endif

	friend FRDGBuilder;
	friend FRDGUserValidation;
	friend FRDGBarrierBatchBegin;
	friend FRDGTrace;
};

/** A render graph resource (e.g. a view) which references a single parent resource (e.g. a texture / buffer). Provides an abstract way to access the parent resource. */
class FRDGView
	: public FRDGResource
{
public:
	/** The type of this child resource; useful for casting between types. */
	const ERDGViewType Type;

	/** Returns the referenced parent render graph resource. */
	virtual FRDGParentResourceRef GetParent() const = 0;

	FRDGViewHandle GetHandle() const
	{
		return Handle;
	}

protected:
	FRDGView(const TCHAR* Name, ERDGViewType InType)
		: FRDGResource(Name)
		, Type(InType)
	{}

private:
	FRDGViewHandle Handle;

	friend FRDGBuilder;
	friend FRDGViewRegistry;
	friend FRDGAllocator;
};

/** Translates from a pooled render target descriptor to an RDG texture descriptor. */
inline FRDGTextureDesc Translate(const FPooledRenderTargetDesc& InDesc, ERenderTargetTexture InTexture = ERenderTargetTexture::Targetable);

/** Translates from an RDG texture descriptor to a pooled render target descriptor. */
inline FPooledRenderTargetDesc Translate(const FRDGTextureDesc& InDesc);

class RENDERCORE_API FRDGPooledTexture final
	: public FRefCountedObject
{
public:
	FRDGPooledTexture(FRHITexture* InTexture, const FRDGTextureSubresourceLayout& InLayout, const FUnorderedAccessViewRHIRef& FirstMipUAV)
		: Texture(InTexture)
		, Layout(InLayout)
	{
		InitViews(FirstMipUAV);
		Reset();
	}

	/** Finds a UAV matching the descriptor in the cache or creates a new one and updates the cache. */
	FRHIUnorderedAccessView* GetOrCreateUAV(const FRHITextureUAVCreateInfo& UAVDesc);

	/** Finds a SRV matching the descriptor in the cache or creates a new one and updates the cache. */
	FRHIShaderResourceView* GetOrCreateSRV(const FRHITextureSRVCreateInfo& SRVDesc);

	FRHITexture* GetRHI() const
	{
		return Texture;
	}

	FRDGTexture* GetOwner() const
	{
		return Owner;
	}

private:
	/** Initializes cached views. Safe to call multiple times; each call will recreate. */
	void InitViews(const FUnorderedAccessViewRHIRef& FirstMipUAV);

	/** Prepares the pooled texture state for re-use across RDG builder instances. */
	void Finalize();

	/** Resets the pooled texture state back an unknown value. */
	void Reset();

	TRefCountPtr<FRHITexture> Texture;
	FRDGTexture* Owner = nullptr;
	FRDGTextureSubresourceLayout Layout;
	FRDGTextureSubresourceState State;

	/** Cached views created for the RHI texture. */
	TArray<FUnorderedAccessViewRHIRef, TInlineAllocator<1>> MipUAVs;
	TArray<TPair<FRHITextureSRVCreateInfo, FShaderResourceViewRHIRef>, TInlineAllocator<1>> SRVs;
	FUnorderedAccessViewRHIRef HTileUAV;
	FShaderResourceViewRHIRef  HTileSRV;
	FUnorderedAccessViewRHIRef StencilUAV;
	FShaderResourceViewRHIRef  FMaskSRV;
	FShaderResourceViewRHIRef  CMaskSRV;

	friend FRDGTexture;
	friend FRDGBuilder;
	friend FRenderTargetPool;
	friend FRDGAllocator;
};

/** Render graph tracked Texture. */
class RENDERCORE_API FRDGTexture final
	: public FRDGParentResource
{
public:
	const FRDGTextureDesc Desc;
	const ERDGTextureFlags Flags;

	//////////////////////////////////////////////////////////////////////////
	//! The following methods may only be called during pass execution.

	/** Returns the allocated pooled render target. */
	UE_DEPRECATED(5.0, "Accessing the underlying pooled render target has been deprecated. Use GetRHI() instead.")
	IPooledRenderTarget* GetPooledRenderTarget() const
	{
		IF_RDG_ENABLE_DEBUG(ValidateRHIAccess());
		return PooledRenderTarget;
	}

	/** Returns the allocated RHI texture. */
	FRHITexture* GetRHI() const
	{
		return static_cast<FRHITexture*>(FRDGResource::GetRHI());
	}

	//////////////////////////////////////////////////////////////////////////

	FRDGTextureHandle GetHandle() const
	{
		return Handle;
	}

	FRDGTextureSubresourceLayout GetSubresourceLayout() const
	{
		return Layout;
	}

	FRDGTextureSubresourceRange GetSubresourceRange() const
	{
		return FRDGTextureSubresourceRange(Layout);
	}

	FRDGTextureSubresourceRange GetSubresourceRangeSRV() const;

private:
	FRDGTexture(const TCHAR* InName, const FRDGTextureDesc& InDesc, ERDGTextureFlags InFlags, ERenderTargetTexture InRenderTargetTexture)
		: FRDGParentResource(InName, ERDGParentResourceType::Texture)
		, Desc(InDesc)
		, Flags(InFlags)
		, RenderTargetTexture(InRenderTargetTexture)
		, Layout(InDesc)
	{
		InitAsWholeResource(MergeState);
		InitAsWholeResource(LastProducers);
	}

	/** Assigns a pooled render target as the backing RHI resource. */
	void SetRHI(FPooledRenderTarget* PooledRenderTarget);

	/** Assigns a pooled buffer as the backing RHI resource. */
	void SetRHI(FRDGPooledTexture* PooledTexture);

	/** Assigns a transient buffer as the backing RHI resource. */
	void SetRHI(FRHITransientTexture* TransientTexture, FRDGAllocator& Allocator);

	/** Finalizes the texture for execution; no other transitions are allowed after calling this. */
	void Finalize();

	/** Returns RHI texture without access checks. */
	FRHITexture* GetRHIUnchecked() const
	{
		return static_cast<FRHITexture*>(FRDGResource::GetRHIUnchecked());
	}

	/** Whether this texture is the last owner of the allocation in the graph. */
	bool IsLastOwner() const
	{
		return NextOwner.IsNull();
	}

	/** Returns the current texture state. Only valid to call after SetRHI. */
	FRDGTextureSubresourceState& GetState()
	{
		check(State);
		return *State;
	}

	/** Describes which RHI texture this RDG texture represents on a pooled texture. Must be default unless the texture is externally registered. */
	const ERenderTargetTexture RenderTargetTexture;

	/** The layout used to facilitate subresource transitions. */
	FRDGTextureSubresourceLayout Layout;

	/** The next texture to own the PooledTexture allocation during execution. */
	FRDGTextureHandle NextOwner;

	/** The handle registered with the builder. */
	FRDGTextureHandle Handle;

	/** The assigned pooled render target to use during execution. Never reset. */
	IPooledRenderTarget* PooledRenderTarget = nullptr;

	union
	{
		/** The assigned pooled texture to use during execution. Never reset. */
		FRDGPooledTexture* PooledTexture = nullptr;

		/** The assigned transient texture to use during execution. Never reset. */
		FRHITransientTexture* TransientTexture;
	};

	/** Cached state pointer from the pooled texture. */
	FRDGTextureSubresourceState* State = nullptr;

	/** Valid strictly when holding a strong reference; use PooledRenderTarget instead. */
	TRefCountPtr<IPooledRenderTarget> Allocation;

	/** Tracks merged subresource states as the graph is built. */
	FRDGTextureTransientSubresourceStateIndirect MergeState;

	/** Tracks pass producers for each subresource as the graph is built. */
	TRDGTextureSubresourceArray<FRDGProducerStatesByPipeline, FRDGArrayAllocator> LastProducers;

#if RDG_ENABLE_DEBUG
	struct FRDGTextureDebugData* TextureDebugData = nullptr;
	FRDGTextureDebugData& GetTextureDebugData() const;
#endif

	friend FRDGBuilder;
	friend FRDGUserValidation;
	friend FRDGBarrierValidation;
	friend FRDGTextureRegistry;
	friend FRDGAllocator;
	friend FPooledRenderTarget;
	friend FRDGTrace;
};

/** Render graph tracked SRV. */
class FRDGShaderResourceView
	: public FRDGView
{
public:
	/** Returns the allocated RHI SRV. */
	FRHIShaderResourceView* GetRHI() const
	{
		return static_cast<FRHIShaderResourceView*>(FRDGResource::GetRHI());
	}

protected:
	FRDGShaderResourceView(const TCHAR* InName, ERDGViewType InType)
		: FRDGView(InName, InType)
	{}

	/** Returns the allocated RHI SRV without access checks. */
	FRHIShaderResourceView* GetRHIUnchecked() const
	{
		return static_cast<FRHIShaderResourceView*>(FRDGResource::GetRHIUnchecked());
	}
};

/** Render graph tracked UAV. */
class FRDGUnorderedAccessView
	: public FRDGView
{
public:
	const ERDGUnorderedAccessViewFlags Flags;

	/** Returns the allocated RHI UAV. */
	FRHIUnorderedAccessView* GetRHI() const
	{
		return static_cast<FRHIUnorderedAccessView*>(FRDGResource::GetRHI());
	}

protected:
	FRDGUnorderedAccessView(const TCHAR* InName, ERDGViewType InType, ERDGUnorderedAccessViewFlags InFlags)
		: FRDGView(InName, InType)
		, Flags(InFlags)
	{}

	/** Returns the allocated RHI UAV without access checks. */
	FRHIUnorderedAccessView* GetRHIUnchecked() const
	{
		return static_cast<FRHIUnorderedAccessView*>(FRDGResource::GetRHIUnchecked());
	}
};

/** Descriptor for render graph tracked SRV. */
class FRDGTextureSRVDesc final
	: public FRHITextureSRVCreateInfo
{
public:
	FRDGTextureSRVDesc() = default;
	
	FRDGTextureRef Texture = nullptr;

	/** Create SRV that access all sub resources of texture. */
	static FRDGTextureSRVDesc Create(FRDGTextureRef Texture)
	{
		FRDGTextureSRVDesc Desc;
		Desc.Texture = Texture;
		Desc.NumMipLevels = Texture->Desc.NumMips;
		return Desc;
	}

	/** Create SRV that access one specific mip level. */
	static FRDGTextureSRVDesc CreateForMipLevel(FRDGTextureRef Texture, int32 MipLevel)
	{
		FRDGTextureSRVDesc Desc;
		Desc.Texture = Texture;
		Desc.MipLevel = MipLevel;
		Desc.NumMipLevels = 1;
		return Desc;
	}

	/** Create SRV that access one specific mip level. */
	static FRDGTextureSRVDesc CreateWithPixelFormat(FRDGTextureRef Texture, EPixelFormat PixelFormat)
	{
		FRDGTextureSRVDesc Desc = FRDGTextureSRVDesc::Create(Texture);
		Desc.Format = PixelFormat;
		return Desc;
	}

	/** Create SRV with access to a specific meta data plane */
	static FRDGTextureSRVDesc CreateForMetaData(FRDGTextureRef Texture, ERDGTextureMetaDataAccess MetaData)
	{
		FRDGTextureSRVDesc Desc = FRDGTextureSRVDesc::Create(Texture);
		Desc.MetaData = MetaData;
		return Desc;
	}
};

/** Render graph tracked SRV. */
class FRDGTextureSRV final
	: public FRDGShaderResourceView
{
public:
	/** Descriptor of the graph tracked SRV. */
	const FRDGTextureSRVDesc Desc;

	FRDGTextureRef GetParent() const override
	{
		return Desc.Texture;
	}

	FRDGTextureSubresourceRange GetSubresourceRange() const;

private:
	FRDGTextureSRV(const TCHAR* InName, const FRDGTextureSRVDesc& InDesc)
		: FRDGShaderResourceView(InName, ERDGViewType::TextureSRV)
		, Desc(InDesc)
	{}

	friend FRDGBuilder;
	friend FRDGViewRegistry;
	friend FRDGAllocator;
};

/** Descriptor for render graph tracked UAV. */
class FRDGTextureUAVDesc final
	: public FRHITextureUAVCreateInfo
{
public:
	FRDGTextureUAVDesc() = default;

	FRDGTextureUAVDesc(FRDGTextureRef InTexture, uint8 InMipLevel = 0, ERHITextureMetaDataAccess InMetaData = ERHITextureMetaDataAccess::None)
		: FRHITextureUAVCreateInfo(InMipLevel, InMetaData)
		, Texture(InTexture)
	{}

	/** Create UAV with access to a specific meta data plane */
	static FRDGTextureUAVDesc CreateForMetaData(FRDGTextureRef Texture, ERDGTextureMetaDataAccess MetaData)
	{
		FRDGTextureUAVDesc Desc = FRDGTextureUAVDesc(Texture, 0);
		Desc.MetaData = MetaData;
		return Desc;
	}

	FRDGTextureRef Texture = nullptr;
};

/** Render graph tracked texture UAV. */
class FRDGTextureUAV final
	: public FRDGUnorderedAccessView
{
public:
	/** Descriptor of the graph tracked UAV. */
	const FRDGTextureUAVDesc Desc;

	FRDGTextureRef GetParent() const override
	{
		return Desc.Texture;
	}

	FRDGTextureSubresourceRange GetSubresourceRange() const;

private:
	FRDGTextureUAV(const TCHAR* InName, const FRDGTextureUAVDesc& InDesc, ERDGUnorderedAccessViewFlags InFlags)
		: FRDGUnorderedAccessView(InName, ERDGViewType::TextureUAV, InFlags)
		, Desc(InDesc)
	{}

	friend FRDGBuilder;
	friend FRDGViewRegistry;
	friend FRDGAllocator;
};

/** Descriptor for render graph tracked Buffer. */
struct FRDGBufferDesc
{
	enum class EUnderlyingType
	{
		VertexBuffer,
		StructuredBuffer
	};

	/** Create the descriptor for an indirect RHI call.
	 *
	 * Note, IndirectParameterStruct should be one of the:
	 *		struct FRHIDispatchIndirectParameters
	 *		struct FRHIDrawIndirectParameters
	 *		struct FRHIDrawIndexedIndirectParameters
	 */
	template<typename IndirectParameterStruct>
	static FRDGBufferDesc CreateIndirectDesc(uint32 NumElements = 1)
	{
		FRDGBufferDesc Desc;
		Desc.UnderlyingType = EUnderlyingType::VertexBuffer;
		Desc.Usage = (EBufferUsageFlags)(BUF_Static | BUF_DrawIndirect | BUF_UnorderedAccess | BUF_ShaderResource);
		Desc.BytesPerElement = sizeof(IndirectParameterStruct);
		Desc.NumElements = NumElements;
		return Desc;
	}

	static FRDGBufferDesc CreateIndirectDesc(uint32 NumElements = 1)
	{
		FRDGBufferDesc Desc;
		Desc.UnderlyingType = EUnderlyingType::VertexBuffer;
		Desc.Usage = (EBufferUsageFlags)(BUF_Static | BUF_DrawIndirect | BUF_UnorderedAccess | BUF_ShaderResource);
		Desc.BytesPerElement = 4;
		Desc.NumElements = NumElements;
		return Desc;
	}

	static FRDGBufferDesc CreateStructuredDesc(uint32 BytesPerElement, uint32 NumElements)
	{
		FRDGBufferDesc Desc;
		Desc.UnderlyingType = EUnderlyingType::StructuredBuffer;
		Desc.Usage = (EBufferUsageFlags)(BUF_Static | BUF_UnorderedAccess | BUF_ShaderResource);
		Desc.BytesPerElement = BytesPerElement;
		Desc.NumElements = NumElements;
		return Desc;
	}

	static FRDGBufferDesc CreateBufferDesc(uint32 BytesPerElement, uint32 NumElements)
	{
		FRDGBufferDesc Desc;
		Desc.UnderlyingType = EUnderlyingType::VertexBuffer;
		Desc.Usage = (EBufferUsageFlags)(BUF_Static | BUF_UnorderedAccess | BUF_ShaderResource);
		Desc.BytesPerElement = BytesPerElement;
		Desc.NumElements = NumElements;
		return Desc;
	}

	static FRDGBufferDesc CreateByteAddressDesc(uint32 NumBytes)
	{
		check(NumBytes % 4 == 0);
		FRDGBufferDesc Desc;
		Desc.UnderlyingType = EUnderlyingType::StructuredBuffer;
		Desc.Usage = (EBufferUsageFlags)(BUF_UnorderedAccess | BUF_ShaderResource | BUF_ByteAddressBuffer);
		Desc.BytesPerElement = 4;
		Desc.NumElements = NumBytes / 4;
		return Desc;
	}

	static FRDGBufferDesc CreateUploadDesc(uint32 BytesPerElement, uint32 NumElements)
	{
		FRDGBufferDesc Desc;
		Desc.UnderlyingType = EUnderlyingType::VertexBuffer;
		Desc.Usage = (EBufferUsageFlags)(BUF_Static | BUF_ShaderResource);
		Desc.BytesPerElement = BytesPerElement;
		Desc.NumElements = NumElements;
		return Desc;
	}

	/** Returns the total number of bytes allocated for a such buffer. */
	uint32 GetTotalNumBytes() const
	{
		return BytesPerElement * NumElements;
	}

	bool operator == (const FRDGBufferDesc& Other) const
	{
		return (
			BytesPerElement == Other.BytesPerElement &&
			NumElements == Other.NumElements &&
			Usage == Other.Usage &&
			UnderlyingType == Other.UnderlyingType);
	}

	bool operator != (const FRDGBufferDesc& Other) const
	{
		return !(*this == Other);
	}

	/** Stride in bytes for index and structured buffers. */
	uint32 BytesPerElement = 1;

	/** Number of elements. */
	uint32 NumElements = 1;

	/** Bitfields describing the uses of that buffer. */
	EBufferUsageFlags Usage = BUF_None;

	/** The underlying RHI type to use. */
	EUnderlyingType UnderlyingType = EUnderlyingType::VertexBuffer;
};

struct FRDGBufferSRVDesc final
	: public FRHIBufferSRVCreateInfo
{
	FRDGBufferSRVDesc() = default;

	FRDGBufferSRVDesc(FRDGBufferRef InBuffer);

	FRDGBufferSRVDesc(FRDGBufferRef InBuffer, EPixelFormat InFormat)
		: FRHIBufferSRVCreateInfo(InFormat)
		, Buffer(InBuffer)
	{
		BytesPerElement = GPixelFormats[Format].BlockBytes;
	}

	FRDGBufferRef Buffer = nullptr;
};

struct FRDGBufferUAVDesc final
	: public FRHIBufferUAVCreateInfo
{
	FRDGBufferUAVDesc() = default;

	FRDGBufferUAVDesc(FRDGBufferRef InBuffer);

	FRDGBufferUAVDesc(FRDGBufferRef InBuffer, EPixelFormat InFormat)
		: FRHIBufferUAVCreateInfo(InFormat)
		, Buffer(InBuffer)
	{}

	FRDGBufferRef Buffer = nullptr;
};

/** Translates from a RDG buffer descriptor to a RHI buffer creation info */
inline FRHIBufferCreateInfo Translate(const FRDGBufferDesc& InDesc);

class RENDERCORE_API FRDGPooledBuffer final
	: public FRefCountedObject
{
public:
	FRDGPooledBuffer(TRefCountPtr<FRHIBuffer> InBuffer, const FRDGBufferDesc& InDesc)
		: Desc(InDesc)
		, Buffer(MoveTemp(InBuffer))
	{}

	const FRDGBufferDesc Desc;

	/** Finds a UAV matching the descriptor in the cache or creates a new one and updates the cache. */
	FRHIUnorderedAccessView* GetOrCreateUAV(const FRHIBufferUAVCreateInfo& UAVDesc);

	/** Finds a SRV matching the descriptor in the cache or creates a new one and updates the cache. */
	FRHIShaderResourceView* GetOrCreateSRV(const FRHIBufferSRVCreateInfo& SRVDesc);

	/** Returns the RHI buffer. */
	FRHIBuffer* GetRHI() const
	{
		return Buffer;
	}

	UE_DEPRECATED(5.0, "Buffers types have been consolidated; use GetRHI() instead.")
	FRHIBuffer* GetVertexBufferRHI() const
	{
		return Buffer;
	}

	UE_DEPRECATED(5.0, "Buffers types have been consolidated; use GetRHI() instead.")
	FRHIBuffer* GetStructuredBufferRHI() const
	{
		return Buffer;
	}

private:
	TRefCountPtr<FRHIBuffer> Buffer;
	TArray<TPair<FRHIBufferUAVCreateInfo, FUnorderedAccessViewRHIRef>, TInlineAllocator<1>> UAVs;
	TArray<TPair<FRHIBufferSRVCreateInfo, FShaderResourceViewRHIRef>, TInlineAllocator<1>> SRVs;

	void Reset()
	{
		Owner = nullptr;
		State = {};
	}

	void Finalize()
	{
		Owner = nullptr;
		State.Finalize();
	}

	const TCHAR* Name = nullptr;
	FRDGBufferRef Owner = nullptr;
	FRDGSubresourceState State;

	uint32 LastUsedFrame = 0;

	friend FRenderGraphResourcePool;
	friend FRDGBuilder;
	friend FRDGBuffer;
	friend FRDGAllocator;
};

/** A render graph tracked buffer. */
class RENDERCORE_API FRDGBuffer final
	: public FRDGParentResource
{
public:
	const FRDGBufferDesc Desc;
	const ERDGBufferFlags Flags;

	//////////////////////////////////////////////////////////////////////////
	//! The following methods may only be called during pass execution.

	/** Returns the underlying RHI buffer resource */
	FRHIBuffer* GetRHI() const
	{
		return static_cast<FRHIBuffer*>(FRDGParentResource::GetRHI());
	}

	/** Returns the buffer to use for indirect RHI calls. */
	FORCEINLINE FRHIBuffer* GetIndirectRHICallBuffer() const
	{
		checkf(Desc.Usage & BUF_DrawIndirect, TEXT("Buffer %s was not flagged for indirect draw usage."), Name);
		return GetRHI();
	}

	/** Returns the buffer to use for RHI calls, eg RHILockVertexBuffer. */
	UE_DEPRECATED(5.0, "Buffers types have been consolidated; use GetRHI() instead.")
	FORCEINLINE FRHIBuffer* GetRHIVertexBuffer() const
	{
		return GetRHI();
	}

	/** Returns the buffer to use for structured buffer calls. */
	UE_DEPRECATED(5.0, "Buffers types have been consolidated; use GetRHI() instead.")
	FORCEINLINE FRHIBuffer* GetRHIStructuredBuffer() const
	{
		return GetRHI();
	}

	//////////////////////////////////////////////////////////////////////////

	FRDGBufferHandle GetHandle() const
	{
		return Handle;
	}

private:
	FRDGBuffer(const TCHAR* InName, const FRDGBufferDesc& InDesc, ERDGBufferFlags InFlags)
		: FRDGParentResource(InName, ERDGParentResourceType::Buffer)
		, Desc(InDesc)
		, Flags(InFlags)
	{}

	/** Assigns a pooled buffer as the backing RHI resource. */
	void SetRHI(FRDGPooledBuffer* InPooledBuffer);

	/** Assigns a transient buffer as the backing RHI resource. */
	void SetRHI(FRHITransientBuffer* InTransientBuffer, FRDGAllocator& Allocator);

	/** Finalizes the buffer for execution; no other transitions are allowed after calling this. */
	void Finalize();

	FRHIBuffer* GetRHIUnchecked() const
	{
		return static_cast<FRHIBuffer*>(FRDGResource::GetRHIUnchecked());
	}

	/** Returns the current buffer state. Only valid to call after SetRHI. */
	FRDGSubresourceState& GetState() const
	{
		check(State);
		return *State;
	}

	/** Registered handle set by the builder. */
	FRDGBufferHandle Handle;

	/** Tracks the last pass that produced this resource as the graph is built. */
	FRDGProducerStatesByPipeline LastProducer;

	/** The next buffer to own the PooledBuffer allocation during execution. */
	FRDGBufferHandle NextOwner;

	union
	{
		/** Assigned pooled buffer pointer. Never reset once assigned. */
		FRDGPooledBuffer* PooledBuffer = nullptr;

		/** Assigned transient buffer pointer. Never reset once assigned. */
		FRHITransientBuffer* TransientBuffer;
	};

	/** Cached state pointer from the pooled / transient buffer. */
	FRDGSubresourceState* State = nullptr;

	/** Valid strictly when holding a strong reference; use PooledBuffer instead. */
	TRefCountPtr<FRDGPooledBuffer> Allocation;

	/** Tracks the merged subresource state as the graph is built. */
	FRDGSubresourceState* MergeState = nullptr;

#if RDG_ENABLE_DEBUG
	struct FRDGBufferDebugData* BufferDebugData = nullptr;
	FRDGBufferDebugData& GetBufferDebugData() const;
#endif

	friend FRDGBuilder;
	friend FRDGBarrierValidation;
	friend FRDGUserValidation;
	friend FRDGBufferRegistry;
	friend FRDGAllocator;
	friend FRDGTrace;
};

/** Render graph tracked buffer SRV. */
class FRDGBufferSRV final
	: public FRDGShaderResourceView
{
public:
	/** Descriptor of the graph tracked SRV. */
	const FRDGBufferSRVDesc Desc;

	FRDGBufferRef GetParent() const override
	{
		return Desc.Buffer;
	}

private:
	FRDGBufferSRV(const TCHAR* InName, const FRDGBufferSRVDesc& InDesc)
		: FRDGShaderResourceView(InName, ERDGViewType::BufferSRV)
		, Desc(InDesc)
	{}

	friend FRDGBuilder;
	friend FRDGViewRegistry;
	friend FRDGAllocator;
};

/** Render graph tracked buffer UAV. */
class FRDGBufferUAV final
	: public FRDGUnorderedAccessView
{
public:
	/** Descriptor of the graph tracked UAV. */
	const FRDGBufferUAVDesc Desc;

	FRDGBufferRef GetParent() const override
	{
		return Desc.Buffer;
	}

private:
	FRDGBufferUAV(const TCHAR* InName, const FRDGBufferUAVDesc& InDesc, ERDGUnorderedAccessViewFlags InFlags)
		: FRDGUnorderedAccessView(InName, ERDGViewType::BufferUAV, InFlags)
		, Desc(InDesc)
	{}

	friend FRDGBuilder;
	friend FRDGViewRegistry;
	friend FRDGAllocator;
};

#include "RenderGraphResources.inl"