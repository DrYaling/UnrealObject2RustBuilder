// Copyright Epic Games, Inc. All Rights Reserved.

/*=============================================================================
	VolumetricRenderTargetViewStatedata.h
=============================================================================*/

#pragma once

#include "CoreMinimal.h"
#include "EngineDefines.h"
#include "RendererInterface.h"
#include "RenderGraphResources.h"

class FRDGBuilder;

class FVolumetricRenderTargetViewStateData
{

public:

	FVolumetricRenderTargetViewStateData();
	~FVolumetricRenderTargetViewStateData();

	void Initialise(
		FIntPoint& ViewRectResolutionIn,
		float UvNoiseScale,
		int32 Mode,
		int32 UpsamplingMode);

	FRDGTextureRef GetOrCreateVolumetricTracingRT(FRDGBuilder& GraphBuilder);
	FRDGTextureRef GetOrCreateVolumetricTracingRTDepth(FRDGBuilder& GraphBuilder);

	FRDGTextureRef GetOrCreateDstVolumetricReconstructRT(FRDGBuilder& GraphBuilder);
	FRDGTextureRef GetOrCreateDstVolumetricReconstructRTDepth(FRDGBuilder& GraphBuilder);

	TRefCountPtr<IPooledRenderTarget> GetDstVolumetricReconstructRT();
	TRefCountPtr<IPooledRenderTarget> GetDstVolumetricReconstructRTDepth();

	FRDGTextureRef GetOrCreateSrcVolumetricReconstructRT(FRDGBuilder& GraphBuilder);
	FRDGTextureRef GetOrCreateSrcVolumetricReconstructRTDepth(FRDGBuilder& GraphBuilder);

	bool GetHistoryValid() const { return bHistoryValid; }
	const FIntPoint& GetCurrentVolumetricReconstructRTResolution() const { return VolumetricReconstructRTResolution; }
	const FIntPoint& GetCurrentVolumetricTracingRTResolution() const { return VolumetricTracingRTResolution; }
	const FIntPoint& GetCurrentTracingPixelOffset() const { return CurrentPixelOffset; }
	const uint32 GetNoiseFrameIndexModPattern() const { return NoiseFrameIndexModPattern; }

	const uint32 GetVolumetricReconstructRTDownsampleFactor() const { return VolumetricReconstructRTDownsampleFactor; }
	const uint32 GetVolumetricTracingRTDownsampleFactor() const { return VolumetricTracingRTDownsampleFactor; }

	FUintVector4 GetTracingCoordToZbufferCoordScaleBias() const;

	float GetUvNoiseScale()		const { return UvNoiseScale; }
	int32 GetMode()				const { return Mode; }
	int32 GetUpsamplingMode()	const { return UpsamplingMode; }

private:

	uint32 VolumetricReconstructRTDownsampleFactor;
	uint32 VolumetricTracingRTDownsampleFactor;

	uint32 CurrentRT;
	bool bFirstTimeUsed;
	bool bHistoryValid;

	int32 FrameId;
	uint32 NoiseFrameIndex;	// This is only incremented once all Volumetric render target samples have been iterated
	uint32 NoiseFrameIndexModPattern;
	FIntPoint CurrentPixelOffset;

	FIntPoint FullResolution;
	FIntPoint VolumetricReconstructRTResolution;
	FIntPoint VolumetricTracingRTResolution;

	static constexpr uint32 kRenderTargetCount = 2;
	TRefCountPtr<IPooledRenderTarget> VolumetricReconstructRT[kRenderTargetCount];
	TRefCountPtr<IPooledRenderTarget> VolumetricReconstructRTDepth[kRenderTargetCount];

	TRefCountPtr<IPooledRenderTarget> VolumetricTracingRT;
	TRefCountPtr<IPooledRenderTarget> VolumetricTracingRTDepth;

	float UvNoiseScale;
	int32 Mode;
	int32 UpsamplingMode;
};


class FTemporalRenderTargetState
{

public:

	FTemporalRenderTargetState();
	~FTemporalRenderTargetState();

	void Initialise(const FIntPoint& ResolutionIn, EPixelFormat FormatIn);

	FRDGTextureRef GetOrCreateCurrentRT(FRDGBuilder& GraphBuilder);
	void ExtractCurrentRT(FRDGBuilder& GraphBuilder, FRDGTextureRef RDGRT);

	FRDGTextureRef GetOrCreatePreviousRT(FRDGBuilder& GraphBuilder);

	bool GetHistoryValid() const { return bHistoryValid; }

	bool CurrentIsValid() const { return RenderTargets[CurrentRT].IsValid(); }
	TRefCountPtr<IPooledRenderTarget> CurrentRenderTarget() const { return RenderTargets[CurrentRT]; }

	uint32 GetCurrentIndex() { return CurrentRT; }
	uint32 GetPreviousIndex() { return 1 - CurrentRT; }

	void Reset();

private:

	uint32 CurrentRT;
	int32 FrameId;

	bool bFirstTimeUsed;
	bool bHistoryValid;

	FIntPoint Resolution;
	EPixelFormat Format;

	static constexpr uint32 kRenderTargetCount = 2;
	TRefCountPtr<IPooledRenderTarget> RenderTargets[kRenderTargetCount];
};



