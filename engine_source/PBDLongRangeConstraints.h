// Copyright Epic Games, Inc. All Rights Reserved.
#pragma once

#include "Chaos/PBDLongRangeConstraintsBase.h"
#include "ChaosStats.h"

DECLARE_CYCLE_STAT(TEXT("Chaos PBD Long Range Constraint"), STAT_PBD_LongRange, STATGROUP_Chaos);

namespace Chaos
{
template<class T, int d>
class CHAOS_API TPBDLongRangeConstraints final : public TPBDLongRangeConstraintsBase<T, d>
{
public:
	typedef TPBDLongRangeConstraintsBase<T, d> Base;
	typedef typename Base::FTether FTether;
	typedef typename Base::EMode EMode;

	using Base::GetMode;

	TPBDLongRangeConstraints(
		const TPBDParticles<T, d>& Particles,
		const TMap<int32, TSet<int32>>& PointToNeighbors,
		const int32 MaxNumTetherIslands = 4,
		const T InStiffness = (T)1,
		const T LimitScale = (T)1,
		const EMode InMode = EMode::Geodesic)
		: TPBDLongRangeConstraintsBase<T, d>(Particles, PointToNeighbors, MaxNumTetherIslands, InStiffness, LimitScale, InMode) {}
	~TPBDLongRangeConstraints() {}

	void Apply(TPBDParticles<T, d>& Particles, const T Dt, const TArray<int32>& ConstraintIndices) const;
	void Apply(TPBDParticles<T, d>& Particles, const T Dt) const;

private:
	using Base::Tethers;
	using Base::TethersView;
	using Base::Stiffness;
};
}

// Support ISPC enable/disable in non-shipping builds
#if !INTEL_ISPC
const bool bChaos_LongRange_ISPC_Enabled = false;
#elif UE_BUILD_SHIPPING
const bool bChaos_LongRange_ISPC_Enabled = true;
#else
extern CHAOS_API bool bChaos_LongRange_ISPC_Enabled;
#endif
